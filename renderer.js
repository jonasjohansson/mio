"use strict";
const { ipcRenderer } = require("electron");
const { BrowserWindow } = require("electron").remote;
const SerialPort = require("serialport");
const Readline = require("@serialport/parser-readline");
const midi = require("midi");
const robot = require("robotjs");
const IO = require("./io");
const config = require("./config");

const output = new midi.Output();
const parser = new Readline();

var port;
let main;
let aside;

let connectBtn, portSelect, baudSelect, devicesSelect;

var portOpen = false;
var midiOpen = false;

const ios = [];

var index = 0;

var keysPressed = [];
const keysAllowed = config.get("keys");

// robot.setKeyboardDelay(10);

document.addEventListener("DOMContentLoaded", () => {
  main = document.querySelector("main");
  aside = document.querySelector("aside");
  connectBtn = document.querySelector("#connect");
  // connectBtn.disabled = true;
  portSelect = document.querySelector("#ports");
  baudSelect = document.querySelector("#baudrates");
  devicesSelect = document.querySelector("#devices");

  /* Get baudrates */
  for (const baudrate of config.get("baudrates")) {
    var option = document.createElement("option");
    option.textContent = baudrate;
    baudSelect.appendChild(option);
  }

  for (const ioData of config.get("ios")) {
    ioData.index = ++index;
    createIO(ioData);
  }

  scan();
  scanMidi();
});

function createIO(data) {
  var io = new IO(data);
  ios.push(io);
  main.appendChild(io.view);
}

function scan() {
  removeAllChildren(portSelect);
  SerialPort.list(function(err, ports) {
    // ports = ports.filter(isArduino);
    // connectBtn.disabled != ports.length;
    for (var port of ports) {
      var option = document.createElement("option");
      option.textContent = port.comName;
      if (port.comName.includes("usbmodem")) {
        option.selected = true;
      }
      portSelect.appendChild(option);
    }
  });
}

function scanMidi() {
  removeAllChildren(devicesSelect);
  var portCount = output.getPortCount();
  for (var i = 0; i < portCount; i++) {
    var option = document.createElement("option");
    option.textContent = output.getPortName(i);
    devicesSelect.appendChild(option);
  }
}

function connect(el) {
  if (portOpen === true) {
    port.close();
    portOpen = false;
    el.textContent = "connect";
    document.documentElement.classList.remove("serial-connected");
  } else if (portSelect.value) {
    port = new SerialPort(portSelect.value, {
      baudRate: Number(baudSelect.value),
      autoOpen: true,
      lock: false
    });
    port.pipe(parser);
    portOpen = true;
    document.documentElement.classList.add("serial-connected");
    el.textContent = "disconnect";
  }
}

function connectMidi(el) {
  if (midiOpen === true) {
    output.closePort(devicesSelect.selectedIndex);
    midiOpen = false;
    el.textContent = "connect";
  } else {
    midiOpen = true;
    output.openPort(devicesSelect.selectedIndex);
    el.textContent = "disconnect";
  }
}

function testMidi() {
  var val = Math.round(Math.random() * 127);
  var msg = [176, 0, val];
  output.sendMessage(msg);
}

parser.on("data", str => {
  // remove whitespace
  str = str.trim();
  if (str.length <= 1) return;

  sendSimple(str);
  sendAdvanced(str);
});

function sendAdvanced(str) {
  // for each io
  for (const io of ios) {
    // get pin
    var pin = io.pins.value;
    if (str.includes(pin)) {
      // remove pin and get value
      var val = str.replace(pin, "");
      // get number from string
      val = parseInt(val);
      // update io with new value
      io.update(val);

      // send key press
      if (io.keySend.checked) {
        var mod = io.keysMod.options[io.keysMod.selectedIndex].value;
        // if output is more than 0
        if (Number(io.output.value) > 0) {
          if (io.keyLong.checked) {
            robot.keyToggle(io.keys.value, "down", mod);
            io.keyPressed = true;
          } else {
            if (io.keyPressed === false) {
              io.keyPressed = true;
              pressKey(io.keys.value, mod);
            }
          }
        } else {
          if (io.keyPressed === true) {
            io.keyPressed = false;
            if (io.keyLong.checked) {
              robot.keyToggle(io.keys.value, "up", mod);
            }
          }
        }
      }

      // send midi key
      if (io.midiSend.checked && val !== io.prev) {
        var val = Number(io.output.value);
        var msg = [io.midiCntrol.value, io.midiChannel.value, val];
        output.sendMessage(msg);
        console.log(`Midi: ${msg}`);
        io.prev = val;
      }
    }
  }
}

function sendSimple(str) {
  var count = (str.match(/(\$)|(\!)+/g) || []).length;
  var key = str.substr(count);
  var first = str[0];
  var index = keysAllowed.indexOf(key);
  if (index > 0) {
    if (first === "$") {
      if (count === 1) {
        if (!keysPressed.includes(key)) {
          pressKey(key);
          keysPressed.push(key);
        }
      } else {
        robot.keyToggle(key, "down");
        if (!keysPressed.includes(key)) {
          keysPressed.push(key);
        }
      }
      // output.sendMessage([176, index, 127]);
    } else if (first === "!") {
      if (count === 1) {
        keysPressed = keysPressed.filter(k => k !== key);
      } else {
        if (keysPressed.includes(key)) {
          robot.keyToggle(key, "up");
        }
        keysPressed = keysPressed.filter(k => k !== key);
      }
      // output.sendMessage([176, index, 0]);
    }
  }
}

function pressKey(key) {
  console.log(`Key: ${key}`);
  robot.keyToggle(key, "down");
  robot.keyToggle(key, "up");
}

function arrayContains(needle, arrhaystack) {
  return arrhaystack.indexOf(needle) > -1;
}

function isArduino(port) {
  var pm = port["manufacturer"];
  return pm !== undefined && pm.includes("arduino");
}

function removeAllChildren(node) {
  while (node.firstChild) {
    node.removeChild(node.firstChild);
  }
}

ipcRenderer.on("save", () => {
  var ioData = [];
  for (var io of ios) {
    ioData.push({
      pin: io.pins.value,
      inMin: Number(io.inMin.value),
      inMax: Number(io.inMax.value),
      outMin: io.outMin,
      outMax: io.outMax,
      // threshold: Number(io.keyThreshold.value),
      key: io.keys.value,
      keySend: io.keySend.checked,
      midiSend: io.midiSend.checked,
      midiChannel: Number(io.midiChannel.value)
    });
  }
  config.set("ios", ioData);
});
