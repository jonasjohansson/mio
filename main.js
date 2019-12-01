"use strict";
const { app, BrowserWindow, Tray, ipcMain } = require("electron");
const electron = require("electron");
const path = require("path");
const url = require("url");
const menu = require("./menu");
const config = require("./config");

const trayIconDefault = `${__dirname}/assets/trayIcon.png`;
const trayIconUnread = `${__dirname}/assets/trayIconUnread.png`;

const width = 200;
const height = 300;

let tray, win;

app.on("ready", () => {
  // app.dock.hide();
  tray = new Tray(trayIconUnread);
  tray.on("click", () => {
    win.show();
    win.focus();
  });
  electron.Menu.setApplicationMenu(menu);
  createWindow();
});

app.on("activate", () => {
  win.show();
});

app.on("before-quit", () => {
  config.set("lastWindowState", win.getBounds());
});

function createWindow() {
  const lastWindowState = config.get("lastWindowState");
  win = new BrowserWindow({
    title: app.getName(),
    x: lastWindowState.x,
    y: lastWindowState.y,
    width: lastWindowState.width,
    height: lastWindowState.height,
    minWidth: width,
    maxWidth: width,
    minHeight: height,
    maxHeight: height,
    // titleBarStyle: "hiddenInset",
    // frame: false,
    alwaysOnTop: config.get("alwaysOnTop")
  });

  win.loadURL(`file://${__dirname}/index.html`);
}

ipcMain.on("quit", () => {
  app.quit();
});

// ipcMain.on("keyUp", () => {
//   // tray.setTitle("");
//   tray.setImage(trayIconDefault);
// });

// ipcMain.on("keyDown", (event, key) => {
//   // tray.setTitle(` ${key}`);
//   tray.setImage(trayIconUnread);
// });
