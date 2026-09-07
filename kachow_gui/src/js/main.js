const menuReceive = document.getElementById("MenuReceive");
const menuSend = document.getElementById("MenuSend");
const deviceID = document.getElementById("DeviceID")
const deviceImage = document.getElementById("DeviceImage")
const deviceImageInput = document.getElementById("DeviceImageInput")
const deviceName= document.getElementById("DeviceName")
const body = document.body;
menuReceive.addEventListener("click", () => {
    body.setAttribute("data-mode", "Receive");
});

menuSend.addEventListener("click", () => {
    body.setAttribute("data-mode", "Send");
});


function loadMyDevice(data) {
    deviceImage.style.backgroundImage = `url('${data.image}')`;
    deviceName.value = data.name;
    deviceID.textContent = `DEVICE ID: ${data.id}`
}

deviceName.addEventListener("blur", ()=>{
    IPC_emit("SetNameDevice", {"name": deviceName.value.trim()});
});

deviceImageInput.addEventListener("change", ()=>{
    if (deviceImageInput.files && deviceImageInput.files[0]) {
        let reader = new FileReader();
        reader.onload = function (e) {
            IPC_emit("SetImageDevice",{"image": e.target.result})
            deviceImage.style.backgroundImage = `url('${e.target.result}')`;
        };
        reader.readAsDataURL(deviceImageInput.files[0]);
    }
});

