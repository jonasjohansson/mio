"use strict";
const electron = require("electron");
const config = require("./config");
const openAboutWindow = require("electron-about-window").default;

const { app, BrowserWindow, shell } = electron;
const appName = app.getName();

function sendAction(action, arg = null) {
  const win = BrowserWindow.getAllWindows()[0];
  win.webContents.send(action, arg);
}

const appMenu = [
  {
    label: "About Mio",
    click: () =>
      openAboutWindow({
        icon_path: `${__dirname}/assets/icon.png`,
        copyright: `Copyright (c) ${new Date().getFullYear()} Jonas Johansson`,
        homepage: "https://jonasjohansson.itch.io/mio",
        win_options: {
          titleBarStyle: "hidden"
          // parent: BrowserWindow.getFocusedWindow(),
          // modal: true,
        },
        // show_close_button: 'Close',
        package_json_dir: __dirname
      })
  },
  { type: "separator" },
  {
    label: "Preferences…",
    accelerator: "Cmd+,",
    click() {
      config.openInEditor();
    }
  },
  {
    label: "Save",
    accelerator: "Cmd+s",
    click() {
      sendAction("save");
    }
  },
  // {
  //   label: "Advanced Mode",
  //   type: "checkbox",
  //   checked: config.get("advancedMode"),
  //   click() {
  //     config.set("advancedMode", !config.get("advancedMode"));
  //   }
  // },
  { type: "separator" },
  { role: "hide" },
  { role: "hideothers" },
  { role: "unhide" },
  { type: "separator" },
  { role: "quit" }
];

const windowMenu = [{ role: "minimize" }, { role: "close" }];

const helpMenu = [
  {
    label: "Website",
    click() {
      shell.openExternal("https://jonasjohansson.se");
    }
  },
  {
    label: "Source Code",
    click() {
      shell.openExternal("https://github.com/jonasjohansson/mio");
    }
  },
  { type: "separator" },
  { role: "toggledevtools" },
  {
    label: "Reset",
    click() {
      config.clear();
    }
  }
];

const menu = [
  {
    label: appName,
    submenu: appMenu
  },
  // {
  //   role: "editMenu"
  // },
  {
    role: "window",
    submenu: windowMenu
  },
  {
    role: "help",
    submenu: helpMenu
  }
];

module.exports = electron.Menu.buildFromTemplate(menu);
