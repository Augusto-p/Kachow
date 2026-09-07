const deviceModePrivate = document.getElementById("DeviceModePrivate");
const deviceModePublic = document.getElementById("DeviceModePublic");
const devicePairCode = document.getElementById("PairCode");
const devicePairQRCode = document.getElementById("PairQRCode");
const publicModeArc = document.getElementById('PublicModeArc');
const downloadsFolder = document.getElementById("DownloadsFolder");
const donloadFolderBtn = document.getElementById("DonloadFolderBtn");
let PublicModeInterval = null;


deviceModePrivate.addEventListener("click", () => {
    deviceModePrivate.parentElement.setAttribute("data-mode", "private");
    publicModeArc.setAttribute("data-value", -1);
    updateArc();
    clearInterval(PublicModeInterval);
    IPC_emit("SetPrivateMode");
});


deviceModePublic.addEventListener("click", () => {
    deviceModePublic.parentElement.setAttribute("data-mode", "public");
    IPC_emit("SetPublicMode");
    
})

donloadFolderBtn.addEventListener('click', async _ => {

    try {
        const absolutePath = await window.__TAURI_PLUGIN_DIALOG__.open({
            directory: true,
            multiple: false,
            title: 'Select Downloads Folder'
        });

        if (absolutePath) {
            IPC_emit("SetDownloadFolder", {"path": absolutePath});
        }
    } catch (error) {
        console.error('Error selecting folder:', error);
    }
});

function loadPairCode(PairCode) {
    let qrInfo = `hello pair code ${PairCode}`;
    devicePairCode.textContent = PairCode;
    const qr = qrcode(0, "M");
    qr.addData(qrInfo);
    qr.make();
    devicePairQRCode.innerHTML = qr.createSvgTag({
        scalable: true,
        padding: 0

    });;
    let svg = devicePairQRCode.firstElementChild;
    svg.removeChild(svg.firstElementChild)
    svg.firstElementChild.removeAttribute("fill")


}

function loadPairMode(data) {
    if (data.mode == "private") {
        deviceModePrivate.parentElement.setAttribute("data-mode", "private");
        publicModeArc.setAttribute("data-value", -1);
        updateArc();
        clearInterval(PublicModeInterval);
    } else {
        const unixSeconds = Math.floor(Date.now() / 1000);
        const value = Math.trunc(unixSeconds - data.time);
        deviceModePublic.parentElement.setAttribute("data-mode", "public");
        publicModeArc.setAttribute("data-value", value);
        updateArc();
        PublicModeInterval = setInterval(() => {
            if (value == 600) {
                publicModeArc.setAttribute("data-value", -1);
                updateArc();
                clearInterval(PublicModeInterval);
                deviceModePrivate.parentElement.setAttribute("data-mode", "private");
                return
            }
            publicModeArc.setAttribute("data-value", value + 1);
            updateArc();
        }, 60000);

    }

}

function loadDonwloadFolder(path) {
    downloadsFolder.textContent = path;

}

function updateArc() {
    const value = parseInt(publicModeArc.dataset.value) ?? -1;
    const degrees = parseInt(360 - value / 6 * 3.6);
    if (value == -1) {
        publicModeArc.firstElementChild.style.opacity = 0;
        publicModeArc.lastElementChild.style.opacity = 0;
        return;
    }
    if (degrees == 360) {
        publicModeArc.firstElementChild.style.opacity = 0;
        publicModeArc.lastElementChild.style.opacity = 1;
        return;
    }
    const r = 104;
    const cx = 110;
    const cy = 110;
    const x2 = cx + r * Math.cos((degrees * Math.PI) / 180);
    const y2 = cy + r * Math.sin((degrees * Math.PI) / 180);

    const largeArcFlag = degrees > 180 ? 1 : 0;
    publicModeArc.firstElementChild.setAttribute(
        'd',
        `M ${cx} ${cy} L 214 110 A ${r} ${r} 0 ${largeArcFlag} 1 ${x2} ${y2} Z`
    );

    publicModeArc.lastElementChild.style.opacity = 0;
    publicModeArc.firstElementChild.style.opacity = 1;
}

updateArc()
