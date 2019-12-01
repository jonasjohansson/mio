const electron = require("electron");
const config = require("./config");
const openAboutWindow = require("electron-about-window").default;

const { app, shell } = electron;

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
  },
  { role: "reload" }
];

const menu = [
  {
    label: app.getName(),
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
