const { app, Menu, Tray, BrowserWindow } = require("electron");
const SerialPort = require("serialport");

function sendAction(action, arg = null) {
  const win = BrowserWindow.getAllWindows()[0];
  win.webContents.send(action, arg);
}

let tray = null;
let portMenu = [];
app.on("ready", () => {
  tray = new Tray("assets/trayIcon.png");
  const contextMenu = Menu.buildFromTemplate([
    {
      label: "Port",
      submenu: portMenu,
      click() {
        config.clear();
      }
    }
  ]);
  tray.setContextMenu(contextMenu);
});

SerialPort.list(function(err, ports) {
  for (var p of ports) {
    portMenu.push({
      label: p.comName,
      type: "radio",
      checked: false,
      click() {
        sendAction("connect", p);
      }
    });
  }
});
