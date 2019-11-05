"use strict";
const { ipcRenderer } = require("electron");
const { BrowserWindow } = require("electron").remote;
const SerialPort = require("serialport");
const Readline = require("@serialport/parser-readline");
const IO = require("./io");
const config = require("./config");
const midi = require("midi");
const robot = require("robotjs");

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
var mode = config.get("mode");

var keys = [];
const lookup = [
  "a",
  "b",
  "c",
  "d",
  "e",
  "f",
  "g",
  "h",
  "i",
  "j",
  "k",
  "l",
  "m",
  "n",
  "o",
  "p",
  "q",
  "r",
  "s",
  "t",
  "u",
  "v",
  "w",
  "x",
  "y",
  "z",
  "alt",
  "ctrl",
  "space",
  "shift",
  "enter",
  "up",
  "down",
  "left",
  "right"
];

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
    ports = ports.filter(isArduino);
    // connectBtn.disabled != ports.length;
    for (var port of ports) {
      var option = document.createElement("option");
      option.textContent = port.comName;
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
    console.log(devicesSelect.selectedIndex);
    el.textContent = "disconnect";
  }
}
parser.on("data", str => {
  // remove whitespace
  str = str.trim();
  if (str.length <= 1) return;

  if (mode === "advanced") {
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
          // if output is more than key press threshold
          if (io.output.value >= io.keyThreshold.value) {
            // and if key is not pressed
            if (io.keyPressed === false) {
              pressKey(io.keys.value);
              io.keyPressed = true;
            }
          } else {
            if (io.keyPressed === true) {
              io.keyPressed = false;
            }
          }
        }
        // send midi key
        if (io.midiSend.checked) {
          output.sendMessage([1, Number(io.output.value), io.id]);
        }
      }
    }
  } else {
    var key = str.substr(1);
    var first = str[0];
    var index = lookup.indexOf(key);
    if (index > 0) {
      if (first === "$") {
        if (!keys.includes(key)) {
          pressKey(key);
          keys.push(key);
          output.sendMessage([1, 127, index]);
        }
      } else if (first === "!") {
        keys = keys.filter(k => k !== key);
        output.sendMessage([1, 0, index]);
      }
    }
  }
});

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
