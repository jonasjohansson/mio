"use strict";

const SerialPort = require("serialport");
const Readline = require("@serialport/parser-readline");
const midi = require("midi");
const WebSocket = require("ws");
const osc = require("osc");
const robot = require("robotjs");
const config = require("./config");

const output = new midi.Output();
const parser = new Readline();
const keysAllowed = config.get("keys");

const remoteServer = config.get("remoteServer");
const wss = remoteServer === "" ? new WebSocket.Server({ port: config.get("port") || 8080 }) : new WebSocket(`wss://${remoteServer}`);

var port;
var oscPort;
var keysPressed = [];
var keysIncoming = [];

var now;
var then = Date.now();
var interval = config.get("interval");
var delta;

var udpPort = new osc.UDPPort({
  localAddress: "0.0.0.0",
  remoteAddress: "0.0.0.0",
  localPort: 7000,
  remotePort: 7001,
  broadcast: true
});
udpPort.open();

let portSelect, baudSelect, devicesSelect, intervalInput;

var portOpen = false;
var midiOpen = false;

document.addEventListener("DOMContentLoaded", () => {
  portSelect = document.querySelector("#ports");
  baudSelect = document.querySelector("#baudrates");
  devicesSelect = document.querySelector("#devices");
  intervalInput = document.querySelector("#interval");

  intervalInput.value = interval;

  /* Get interval rate */
  intervalInput.addEventListener("change", function() {
    interval = Number(intervalInput.value);
    config.set("interval", interval);
  });

  /* Get baudrates */
  for (const baudrate of config.get("baudrates")) {
    var option = document.createElement("option");
    option.textContent = baudrate;
    baudSelect.appendChild(option);
  }

  scan();
  scanMidi();
});

function scan() {
  var connectBtn = document.querySelector("#connect");
  removeAllChildren(portSelect);
  SerialPort.list(function(err, ports) {
    // ports = ports.filter(isArduino);
    connectBtn.disabled = !ports.length;
    for (var port of ports) {
      console.log(port);
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
  var connectBtn = document.querySelector("#connectMidi");
  removeAllChildren(devicesSelect);
  var portCount = output.getPortCount();
  connectBtn.disabled = !portCount;
  for (var i = 0; i < portCount; i++) {
    var option = document.createElement("option");
    option.textContent = output.getPortName(i);
    devicesSelect.appendChild(option);
  }
}

function connect(el) {
  if (portOpen === false) {
    port = new SerialPort(portSelect.value, {
      baudRate: Number(baudSelect.value),
      autoOpen: true,
      lock: false
    });
    port.on("open", function() {
      document.documentElement.classList.add("serial-connected");
      el.textContent = "disconnect";
      portOpen = true;
    });
    port.on("close", function() {
      document.documentElement.classList.remove("serial-connected");
      el.textContent = "connect";
      portOpen = false;
    });
    port.on("error", err => {
      alert(err);
    });
    port.pipe(parser);
  } else {
    port.close();
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
    output.sendMessage([176, 44, 127]);
    output.sendMessage([16, 1, 0]);
  }
}

parser.on("data", str => {
  onData(str);
});

function onData(str) {
  str = str.toLowerCase();
  str = str.trim();

  var first = str[0];

  // key logic
  if (first === "$" || first === "!") {
    var key = str.substr(1);
    var keyIndex = getIndex(key);

    if (keyIndex < 0) return;

    if (first === "$") {
      if (!keysPressed.includes(key)) {
        keysPressed.push(key);
        robot.keyToggle(key, "down");
        output.sendMessage([16, 127, keyIndex]);
        log(`${key}: ↓`);
      }
    } else {
      if (keysPressed.includes(key)) {
        keysPressed = keysPressed.filter(k => k !== key);
        robot.keyToggle(key, "up");
        output.sendMessage([16, 0, keyIndex]);
        log(`${key}: ↑`);
      }
    }
    if (!keysIncoming.includes(key)) {
      keysIncoming.push(key);
    }
    return;
  }

  // mouse logic
  for (let mouseEvent of ["movemousesmooth", "movemouse", "mouseclick", "mousetoggle", "dragmouse", "scrollmouse"]) {
    if (str.includes(mouseEvent)) {
      str = str.replace(mouseEvent, "");
      str = str.replace(/\(|\)/g, "");
      var val = str.split(",");
      if (val.length === 2) {
        let a = Number(val[0]);
        let b = Number(val[1]);
        switch (mouseEvent) {
          case "movemousesmooth":
            robot.moveMouseSmooth(a, b);
            break;
          case "movemouse":
            robot.moveMouse(a, b);
            break;
          case "mouseclick":
            robot.mouseClick(val[0], val[0]);
            break;
          case "mousetoggle":
            robot.mouseToggle(val[0], val[0]);
            break;
          case "dragmouse":
            robot.dragMouse(a, b);
            break;
          case "scrollmouse":
            robot.scrollMouse(a, b);
            break;
        }
      }
    }
  }

  // socket logic sensor123
  var data = str.split(/([0-9]+)/).filter(Boolean);
  if (data.length === 2) {
    var dataObject = {
      id: data[0],
      msg: Number(data[1])
    };
    dataObject = JSON.stringify(dataObject);
    wss.clients.forEach(function each(client) {
      if (client.readyState === WebSocket.OPEN) {
        client.send(dataObject);
      }
    });
    port.write(str);
    return;
  }

  // osc logic 123/address/
  var data = str.split(/([0-9\.]+)/).filter(Boolean);
  if (data.length === 2) {
    var dataObject = {
      address: data[0],
      args: new Array({
        type: "f",
        value: Number(data[1])
      })
    };
    udpPort.send(dataObject, "127.0.0.1", 7001);
  }
}

function render() {
  requestAnimationFrame(render);

  now = Date.now();
  delta = now - then;

  if (delta > interval) {
    if (keysPressed.length > 0) {
      for (let key of keysPressed) {
        var keyIndex = getIndex(key);

        // if the key is not incoming any more, it must have been released
        if (!keysIncoming.includes(key)) {
          if (key === "mouse") {
            robot.mouseToggle("up");
          } else {
            robot.keyToggle(key, "up");
          }
          log(`${key}: ↑`);
          var keyIndex = getIndex(key);
          output.sendMessage([16, 0, keyIndex]);
          keysPressed = keysPressed.filter(k => k !== key);
        }
      }
    }
    keysIncoming = [];
    then = now - (delta % interval);
  }
}

requestAnimationFrame(render);

function getIndex(key) {
  return keysAllowed.indexOf(key);
}

function isArduino(port) {
  var p = port["vendorId"];
  return p !== undefined && p.includes("2341");
}

function removeAllChildren(node) {
  while (node.firstChild) {
    node.removeChild(node.firstChild);
  }
}

function log(msg) {
  var log = document.getElementById("log");
  var p = document.createElement("p");
  p.innerHTML = msg;
  log.insertBefore(p, log.firstChild);
  // log.innerHTML = msg;
  console.log(msg);
}

wss.on("connection", socket => {
  console.log("connected");
  socket.on("message", function incoming(message) {
    onData(message);
    console.log("received: %s", message);
  });
});
