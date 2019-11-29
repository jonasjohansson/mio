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
let footer;

let connectBtn, portSelect, baudSelect, devicesSelect;

var portOpen = false;
var midiOpen = false;

const ios = [];

var index = 0;

var keysPressed = [];
var keysIncoming = [];
var keysHold = [];
const keysAllowed = config.get("keys");

// robot.setKeyboardDelay(10);

document.addEventListener("DOMContentLoaded", () => {
  main = document.querySelector("main");
  aside = document.querySelector("aside");
  footer = document.querySelector("footer");
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
    document.documentElement.classList.remove("midi-connected");
  } else {
    midiOpen = true;
    output.openPort(devicesSelect.selectedIndex);
    document.documentElement.classList.add("midi-connected");
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

function sendSimple(str) {
  if (str[0] !== "$") return;

  var count = (str.match(/(\$)|(\!)+/g) || []).length;
  var key = str.substr(count);
  var index = getIndex(key);
  if (index > 0) {
    if (!keysHold.includes(key)) {
      log(`HOLD: ${key}`);
      keysHold.push(key);
      robot.keyToggle(key, "down");
      log(`MIDI ON: ${index}`);
      output.sendMessage([16, 127, index]);
    }
    if (!keysIncoming.includes(key)) {
      keysIncoming.push(key);
    }
  }
}

function sendAdvanced(str) {
  // for each io
  for (const io of ios) {
    // get id
    var id = io.ids.value;
    if (str.includes(id)) {
      // remove id and get value
      var val = str.replace(id, "");
      // get number from string
      val = parseInt(val);
      // update io with new value
      io.update(val);

      // send key press
      if (io.keySend.checked) {
        var mod = io.keysMod.options[io.keysMod.selectedIndex].value;
        // if output is more than 0
        if (Number(io.output.value) > 0) {
          if (io.keyHold.checked) {
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
            if (io.keyHold.checked) {
              robot.keyToggle(io.keys.value, "up", mod);
            }
          }
        }
      }

      // send midi key
      if (io.midiSend.checked && val !== io.prev) {
        var val = Number(io.output.value);
        var msg = [io.midiControl.value, io.midiChannel.value, val];
        output.sendMessage(msg);
        console.log(`Midi: ${msg}`);
        io.prev = val;
      }
    }
  }
}

function pressKey(key, index = 0) {
  robot.keyToggle(key, "down");
  setTimeout(() => {
    robot.keyToggle(key, "up");
    output.sendMessage([176, 1, 0]);
  }, 100);
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

function getIndex(key) {
  return keysAllowed.indexOf(key);
}

function log(str) {
  var line = document.createElement("p");
  line.textContent = str;
  footer.insertBefore(line, footer.firstChild);
}

ipcRenderer.on("save", () => {
  var ioData = [];
  for (var io of ios) {
    ioData.push({
      id: io.ids.value,
      key: io.keys.value,
      keySend: io.keySend.checked,
      keyHold: io.keySend.checked,
      keyMod: io.keyMod.value
      // midiSend: io.midiSend.checked,
      // midiChannel: Number(io.midiChannel.value)
    });
  }
  config.set("ios", ioData);
});

var fps = 30;
var now;
var then = Date.now();
var interval = 1000 / fps;
var delta;

function loop() {
  requestAnimationFrame(loop);
  now = Date.now();
  delta = now - then;

  if (delta > interval) {
    for (var i = 0; i < keysHold.length; i++) {
      var key = keysHold[i];

      // if the key is not incoming any more, it must have been released
      if (!keysIncoming.includes(key)) {
        // the key is released
        keysHold = keysHold.filter(k => k !== key);

        log(`RELEASE: ${key}`);
        robot.keyToggle(key, "up");
        var index = getIndex(key);
        log(`MIDI ON: ${index}`);
        output.sendMessage([16, 127, index]);
      }
    }
    keysIncoming = [];

    then = now - (delta % interval);
  }
}
requestAnimationFrame(loop);
