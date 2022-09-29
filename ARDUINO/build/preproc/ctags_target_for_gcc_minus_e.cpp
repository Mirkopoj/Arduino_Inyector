# 1 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
# 2 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 2
# 10 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
bool spi_string_valido(char str[10], uint32_t *commando_ret);
uint16_t spi_write(uint16_t msgF);

void setup() {
 pinMode(10, 0x1);
 pinMode(2, 0x1);
 pinMode(3, 0x1);
 pinMode(8, 0x1);
 pinMode(9, 0x1);
 digitalWrite(10, 0x1);
 digitalWrite(8, 0x1);
 digitalWrite(9, 0x1);
 SPI.beginTransaction(SPISettings(1000000, 1, 0x04));
 SPI.begin();
 Serial.begin(9600);
}

void loop() {
 char buffer[10];
 int cant = 0;
 uint8_t cont = 0;
 uint32_t commando;

 while (true) {
  while (Serial.available() && cant < 10) {
   buffer[cant] = Serial.read();
   cant++;
  }

  if (cant >= 10) {
   if (spi_string_valido(buffer, &commando)){
    uint16_t ret;
    uint8_t instruccion = commando>>24;
    instruccion &= 0x7F;
    if (instruccion == 0x25) {
     digitalWrite(3,0x1);
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
    digitalWrite(2,0x1);
    commando &= 0xFFFF0000;
    Serial.println(commando|ret);
   }
   cant = 0;
   cont++;
   for (int i=0;i<10;i++) buffer[i] = 'i';
  }
 }
}

bool spi_string_valido(char str[10], uint32_t *commando_ret){
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

uint32_t command_parse(char str[10]){
 uint32_t commando = 0;
 for (int i=0;i<10;i++){
  commando *= 10;
  commando += str[i]-48;
 }
 return commando;
}

uint16_t spi_write(uint16_t msg){
 uint16_t ret;
 digitalWrite(10, 0x0);
 ret = SPI.transfer16(msg);
 digitalWrite(10, 0x1);
 return ret;
}
