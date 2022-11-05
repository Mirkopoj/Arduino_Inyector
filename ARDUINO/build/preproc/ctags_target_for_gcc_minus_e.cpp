# 1 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
# 2 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 2
# 22 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
bool spi_string_valido(char str[10], uint32_t *commando_ret);
uint16_t spi_write(uint16_t msgF);

void setup() {
pinMode(53, 0x1);
pinMode(49, 0x1);
pinMode(48, 0x1);
pinMode(22, 0x1);
pinMode(23, 0x1);
pinMode(24, 0x1);
pinMode(25, 0x1);
pinMode(26, 0x1);
pinMode(27, 0x1);
pinMode(28, 0x1);
pinMode(29, 0x1);
pinMode(2, 0x1);
pinMode(3, 0x1);
pinMode(4, 0x1);
pinMode(5, 0x1);
pinMode(6, 0x1);
pinMode(7, 0x1);
pinMode(8, 0x1);
pinMode(9, 0x1);
digitalWrite(53, 0x1);
SPI.beginTransaction(SPISettings(1000000, 1, 0x04));
SPI.begin();
Serial.begin(9600);


# 50 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
__asm__ __volatile__ ("cli" ::: "memory")
# 50 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
    ;//stop interrupts

//set timer1 

# 53 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint8_t *)(0x80)) 
# 53 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
      = 0;// set entire TCCR1A register to 0

# 54 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint8_t *)(0x81)) 
# 54 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
      = 0;// same for TCCR1B

# 55 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint16_t *)(0x84)) 
# 55 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
      = 0;//initialize counter value to 0
// enable timer compare interrupt

# 57 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint8_t *)(0x6F)) 
# 57 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
      |= (1 << 
# 57 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
               1
# 57 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
                     );

//set timer3 

# 60 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint8_t *)(0x90)) 
# 60 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
      = 0;// set entire TCCR3A register to 0

# 61 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint8_t *)(0x91)) 
# 61 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
      = 0;// same for TCCR3B

# 62 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint16_t *)(0x94)) 
# 62 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
      = 0;//initialize counter value to 0
// enable timer compare interrupt

# 64 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint8_t *)(0x71)) 
# 64 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
      |= (1 << 
# 64 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
               1
# 64 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
                     );

 
# 66 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
__asm__ __volatile__ ("sei" ::: "memory")
# 66 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
     ;//allow interrupts
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
     int pin_state = (commando&0x0000FFFF)==0x0000FFFF? 0x1:0x0;
     digitalWrite(pin, pin_state);
     ret = digitalRead(pin)==0x1? 0xFFFF : 0x0000;
    }
    if (instruccion == 0x37) {
     int pin = (commando>>16)&0xFF;
     int timer = (commando&0x0000FFFF);
     if (pin) { lanzo_tnr(timer); }
     else { lanzo_power_enable(timer); }
    }
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
  case 0x37:
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
 digitalWrite(53, 0x0);
 ret = SPI.transfer16(msg);
 digitalWrite(53, 0x1);
 return ret;
}

void lanzo_tnr(uint16_t timer){
 if (timer == 0) {
  digitalWrite(49,0x0);
  return;
 }
 if (timer == 0xFFFF) {
  digitalWrite(49,0x1);
  return;
 }
 
# 158 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint16_t *)(0x88)) 
# 158 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
      = timer;
 
# 159 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint8_t *)(0x81)) 
# 159 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
       = 1;
 digitalWrite(49,0x1);
}


# 163 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
extern "C" void __vector_17 (void) __attribute__ ((signal,used, externally_visible)) ; void __vector_17 (void)
# 163 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
                     {
 digitalWrite(49,0x0);
 
# 165 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint8_t *)(0x81)) 
# 165 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
       = 0;
 
# 166 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint16_t *)(0x84)) 
# 166 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
       = 0;
}

void lanzo_power_enable(uint16_t timer){
 if (timer == 0) {
  digitalWrite(48,0x0);
  return;
 }
 if (timer == 0xFFFF) {
  digitalWrite(48,0x1);
  return;
 }
 
# 178 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint16_t *)(0x98)) 
# 178 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
      = timer;
 
# 179 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint8_t *)(0x91)) 
# 179 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
       = 1;
 digitalWrite(48,0x1);
}


# 183 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
extern "C" void __vector_32 (void) __attribute__ ((signal,used, externally_visible)) ; void __vector_32 (void)
# 183 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
                     {
 digitalWrite(48,0x0);
 
# 185 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint8_t *)(0x91)) 
# 185 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
       = 0;
 
# 186 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino" 3
(*(volatile uint16_t *)(0x94)) 
# 186 "/home/mirko/MPLABXProjects/SSPA.X/arduino_inyector/arduino_frontend/ARDUINO/ARDUINO.ino"
       = 0;
}
