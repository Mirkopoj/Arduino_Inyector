#include <SPI.h>

#define MSG_MAX 10

#define SS 53
#define MOSI 51
#define MISO 50
#define SCK 52

#define TNR 49
#define POWERENABLE 48

#define RFFWD 22
#define RFRFL 23
#define RFINPUT 24
#define TEMPERATURE 25
#define GAN1CURRENT 26
#define GAN2CURRENT 27
#define GAN3CURRENT 28
#define GAN4CURRENT 29

bool spi_string_valido(char str[MSG_MAX], uint32_t *commando_ret);
uint16_t spi_write(uint16_t msgF);

void setup() {
pinMode(SS, OUTPUT);
pinMode(TNR, OUTPUT);
pinMode(POWERENABLE, OUTPUT);
pinMode(RFFWD, OUTPUT);
pinMode(RFRFL, OUTPUT);
pinMode(RFINPUT, OUTPUT);
pinMode(TEMPERATURE, OUTPUT);
pinMode(GAN1CURRENT, OUTPUT);
pinMode(GAN2CURRENT, OUTPUT);
pinMode(GAN3CURRENT, OUTPUT);
pinMode(GAN4CURRENT, OUTPUT);
pinMode(2, OUTPUT);
pinMode(3, OUTPUT);
pinMode(4, OUTPUT);
pinMode(5, OUTPUT);
pinMode(6, OUTPUT);
pinMode(7, OUTPUT);
pinMode(8, OUTPUT);
pinMode(9, OUTPUT);
digitalWrite(SS, HIGH);
SPI.beginTransaction(SPISettings(1000000, MSBFIRST, SPI_MODE1));
SPI.begin();
Serial.begin(9600);

cli();//stop interrupts

//set timer1 
TCCR1A = 0;// set entire TCCR1A register to 0
TCCR1B = 0;// same for TCCR1B
TCNT1  = 0;//initialize counter value to 0
// enable timer compare interrupt
TIMSK1 |= (1 << OCIE1A);

//set timer3 
TCCR3A = 0;// set entire TCCR3A register to 0
TCCR3B = 0;// same for TCCR3B
TCNT3  = 0;//initialize counter value to 0
// enable timer compare interrupt
TIMSK3 |= (1 << OCIE3A);

	sei();//allow interrupts
}

void loop() {
	char buffer[MSG_MAX];
	int cant = 0;
	uint8_t cont = 0;
	uint32_t commando;

	while (true) {
		while (Serial.available() && cant < MSG_MAX) {
			buffer[cant] = Serial.read();
			cant++;
		}

		if (cant >= MSG_MAX) {
			if (spi_string_valido(buffer, &commando)){
				uint16_t ret;
				uint8_t instruccion = commando>>24;
				instruccion &= 0x7F;
				if (instruccion == 0x25) {
					spi_write((commando>>16)&0x0000FFFF);
					spi_write((commando)&0x0000FFFF);
					ret = spi_write(0);
				}
				if (instruccion == 0x3C) {
					spi_write((commando>>16)&0x0000FFFF);
					ret = spi_write(0);
				}
				if (instruccion == 0x29) {
					int pin = (commando>>16)&0xFF;
					int pin_state = (commando&0x0000FFFF)==0x0000FFFF? HIGH:LOW;
					digitalWrite(pin, pin_state);
					ret = digitalRead(pin)==HIGH? 0xFFFF : 0x0000;
				}
				if (instruccion == 0x37) {
					int pin = (commando>>16)&0xFF;
					int timer = (commando&0x0000FFFF);
					if (pin) { lanzo_tnr(timer);		 }
					else { lanzo_power_enable(timer); }
				}
				commando &= 0xFFFF0000;
				Serial.println(commando|ret);
			}
			cant = 0;
			cont++;
			for (int i=0;i<MSG_MAX;i++) buffer[i] = 'i';
		}
	}
}

bool spi_string_valido(char str[MSG_MAX], uint32_t *commando_ret){
	uint32_t commando = 0;
	commando = command_parse(str);
	switch ((commando>>24)&0x7F) {
		case 0x25:
		case 0x3C:
		case 0x29:
		case 0x37:
			*commando_ret = commando;
			return true;
		default:
			return false;
	}
}

uint32_t command_parse(char str[MSG_MAX]){
	uint32_t commando = 0;
	for (int i=0;i<MSG_MAX;i++){
		commando *= 10;
		commando += str[i]-48;
	}
	return commando;
}

uint16_t spi_write(uint16_t msg){
	uint16_t ret;
	digitalWrite(SS, LOW);
	ret = SPI.transfer16(msg);
	digitalWrite(SS, HIGH);
	return ret;
}

void lanzo_tnr(uint16_t timer){
	if (timer == 0) {
		digitalWrite(TNR,LOW);
		return;
	}
	if (timer == 0xFFFF) {
		digitalWrite(TNR,HIGH);
		return;
	}
	OCR1A = timer;
	TCCR1B = 1;
	digitalWrite(TNR,HIGH);
}

ISR(TIMER1_COMPA_vect){
	digitalWrite(TNR,LOW);
	TCCR1B = 0;
	TCNT1  = 0;
}

void lanzo_power_enable(uint16_t timer){
	if (timer == 0) {
		digitalWrite(POWERENABLE,LOW);
		return;
	}
	if (timer == 0xFFFF) {
		digitalWrite(POWERENABLE,HIGH);
		return;
	}
	OCR3A = timer;
	TCCR3B = 1;
	digitalWrite(POWERENABLE,HIGH);
}

ISR(TIMER3_COMPA_vect){
	digitalWrite(POWERENABLE,LOW);
	TCCR3B = 0;
	TCNT3  = 0;
}
