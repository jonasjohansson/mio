const electron = require('electron');
const config = require('./config');
const openAboutWindow = require('electron-about-window').default;

const { app, shell, mainWindow, BrowserWindow } = electron;

let win;

const appMenu = [
  {
    label: 'About Mio',
    click: () =>
      openAboutWindow({
        icon_path: `${__dirname}/assets/icon.png`,
        copyright: `Copyright (c) ${new Date().getFullYear()} Jonas Johansson`,
        homepage: 'https://jonasjohansson.itch.io/mio',
        win_options: {
          titleBarStyle: 'hidden'
        },
        package_json_dir: __dirname
      })
  },
  { type: 'separator' },
  {
    label: 'Preferences…',
    accelerator: 'Cmd+,',
    click() {
      config.openInEditor();
    }
  },
  {
    label: 'Ghost',
    accelerator: 'Cmd+G',
    type: 'checkbox',
    checked: false,
    click: function (item, BrowserWindow) {
      if (item.checked) {
        BrowserWindow.setOpacity(0);
      } else {
        BrowserWindow.setOpacity(1);
      }
    }
  },
  { type: 'separator' },
  { role: 'hide' },
  { role: 'hideothers' },
  { role: 'unhide' },
  { type: 'separator' },
  { role: 'quit' }
];

const windowMenu = [{ role: 'minimize' }, { role: 'close' }];

const helpMenu = [
  {
    label: 'Website',
    click() {
      shell.openExternal('https://jonasjohansson.se');
    }
  },
  {
    label: 'Source Code',
    click() {
      shell.openExternal('https://github.com/jonasjohansson/mio');
    }
  },
  { type: 'separator' },
  {
    label: 'Open Developer Tools',
    click() {
      win = BrowserWindow.getAllWindows()[0];
      win.webContents.openDevTools({ mode: 'detach' });
    }
  },
  {
    label: 'Reset',
    click() {
      config.clear();
      win = BrowserWindow.getAllWindows()[0];
      win.webContents.session.clearCache(function () {});
    }
  },
  { role: 'reload' }
];

const menu = [
  {
    label: app.name,
    submenu: appMenu
  },
  {
    role: 'window',
    submenu: windowMenu
  },
  {
    role: 'help',
    submenu: helpMenu
  }
];

module.exports = electron.Menu.buildFromTemplate(menu);
