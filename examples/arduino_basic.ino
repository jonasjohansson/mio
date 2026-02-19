// Mio v2 — Arduino Basic Example
//
// This sketch demonstrates the Mio serial protocol.
// Connect via USB and open the serial monitor to see what's sent.
// Mio will parse each line and bridge it to the appropriate output.
//
// Protocol format: PREFIX:SUBCOMMAND,arg1,arg2,...
// Each command must be terminated with a newline (Serial.println does this).

const int BUTTON_PIN = 2;
const int POT_PIN = A0;

bool lastButtonState = HIGH;

void setup() {
  Serial.begin(9600);
  pinMode(BUTTON_PIN, INPUT_PULLUP);
}

void loop() {
  // --- Button: tap a key when pressed ---
  bool buttonState = digitalRead(BUTTON_PIN);
  if (buttonState == LOW && lastButtonState == HIGH) {
    // Button just pressed — tap the space key
    Serial.println("key:tap,space");

    // Also send a MIDI note
    Serial.println("midi:note_on,60,127,0");
  }
  if (buttonState == HIGH && lastButtonState == LOW) {
    // Button released — stop the MIDI note
    Serial.println("midi:note_off,60,0,0");
  }
  lastButtonState = buttonState;

  // --- Potentiometer: send value over WebSocket and OSC ---
  int potValue = analogRead(POT_PIN);

  // Broadcast to WebSocket clients as JSON: {"id":"pot","value":512}
  Serial.print("ws:pot,");
  Serial.println(potValue);

  // Send as OSC message to /sensor/pot
  Serial.print("osc:/sensor/pot,");
  Serial.println(potValue);

  // --- Key hold example (uncomment to test) ---
  // To hold a key down continuously, send key:down every loop iteration.
  // Mio's watchdog will auto-release it if you stop sending.
  //
  // if (buttonState == LOW) {
  //   Serial.println("key:down,space");
  // }

  delay(50);
}
