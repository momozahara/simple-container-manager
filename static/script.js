const labelStatus = document.getElementById("status");
const buttonStart = document.getElementById("start");
const buttonStop = document.getElementById("stop");
let interval;

function getStatus() {
  console.log("interval");
  fetch("/api/json",
    { method: "GET", mode: "cors" })
    .then((response) => {
      const _status = response.status;
      if (_status !== 200) {
        throw new Error();
      }
      return response.json();
    })
    .then((data) => {
      const _status = data.State.Status;
      labelStatus.innerText = _status;
      switch (_status) {
        case "running": {
          buttonStart.disabled = true;
          buttonStop.disabled = false;
          break;
        }
        case "exited": {
          buttonStart.disabled = false;
          buttonStop.disabled = true;
          break;
        }
      }
    })
    .catch(() => {
      labelStatus.innerText = "error";
    });
}

function onStart() {
  fetch("/api/start",
    { method: "POST", mode: "cors" })
    .finally(() => {
      getStatus();
      clearInterval(interval);
      interval = setInterval(getStatus, 6000);
    });
}

function onStop() {
  fetch("/api/stop",
    { method: "POST", mode: "cors" })
    .finally(() => {
      getStatus();
      clearInterval(interval);
      interval = setInterval(getStatus, 6000);
    });
}

function onLoad() {
  getStatus();
  interval = setInterval(getStatus, 6000);
}

document.onload += onLoad();
