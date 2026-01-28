export async function minimizeWindow() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().minimize();
}

export async function toggleMaximizeWindow() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  const win = getCurrentWindow();
  const maximized = await win.isMaximized();
  if (maximized) {
    await win.unmaximize();
  } else {
    await win.maximize();
  }
}

export async function closeWindow() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().close();
}
