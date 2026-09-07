const handlers = {
  "InfoMyDevice": ({ device_id, device_image, device_name, download_dir }) => {
    loadMyDevice({
      "name": device_name,
      "id": device_id,
      "image": device_image
    });
    loadDonwloadFolder(download_dir);
  },
  "LoadDownloadFolder": ({ download_dir }) => {
    loadDonwloadFolder(download_dir);
  },
  "LoadPairCode": ({pair_code, time})=>{
    loadPairMode({"mode": "public", "time": time});
    loadPairCode(pair_code);
  }

};

const { event: TAURI_EVENT } = window.__TAURI__;

TAURI_EVENT.listen("ipc", (event) => {
  console.log("📥 IPC:", event.payload);
  const { name, data } = event.payload;

  handlers?.[name]?.(data);

});

async function IPC_emit(name, data = {}) {
  await TAURI_EVENT.emit("IPC", {
    name,
    data,
  });
}
IPC_emit("Start")