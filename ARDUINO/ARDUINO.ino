#include <SPI.h>

#define MSG_MAX 10

#define SS 10
#define MOSI 11
#define MISO 12
#define SCK 13

bool spi_string_valido(char str[MSG_MAX], uint32_t *commando_ret);
uint16_t spi_write(uint16_t msgF);

void setup() {
	pinMode(SS, OUTPUT);
	pinMode(2, OUTPUT);
	pinMode(3, OUTPUT);
	digitalWrite(SS, HIGH);
	SPI.begin();
	Serial.begin(9600);
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
					digitalWrite(3,HIGH);
					spi_write((commando>>16)&0x0000FFFF);
					spi_write((commando)&0x0000FFFF);
					ret = spi_write(0);
				}
				/*switch (instruccion) {
					case 0x29:
						digitalWrite(3,HIGH);
						int pin = (commando>>16)&0xFF;
						int pin_state = (commando&0x0000FFFF)==0x0000FFFF? HIGH:LOW;
						digitalWrite(pin, pin_state);
						ret = digitalRead(pin)==HIGH? 0xFFFF : 0x0000;
						break;
					case 0x3C:
						digitalWrite(3,HIGH);
						spi_write((commando>>16)&0x0000FFFF);
						ret = spi_write(0);
						break;
					case 0x25:
						digitalWrite(3,HIGH);
						spi_write((commando>>16)&0x0000FFFF);
						spi_write((commando)&0x0000FFFF);
						ret = spi_write(0);
						break;
					default:
						digitalWrite(3,HIGH);
						break;
				}*/
				digitalWrite(2,HIGH);
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
