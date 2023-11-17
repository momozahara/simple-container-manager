const labelStatus = document.getElementById("status");
const logsContainer = document.getElementById("logs-container");
const textLogs = document.getElementById("logs");
const buttonStart = document.getElementById("start");
const buttonStop = document.getElementById("stop");
let interval;
let intervalLogs;

function getStatus() {
  fetch("/api/json",
    { method: "GET" })
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

async function getLogs() {
  return fetch("/api/logs",
    { method: "GET" })
    .then((response) => {
      const _status = response.status;
      if (_status !== 200) {
        throw new Error();
      }
      return response.text();
    })
    .then((data) => {
      textLogs.innerText = data;
    })
    .catch(() => {
      textLogs.innerText = "error";
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

async function onLoad() {
  getStatus();
  interval = setInterval(getStatus, 6000);
  await getLogs();
  intervalLogs = setInterval(getLogs, 10000);
  logsContainer.scrollTo(0, logsContainer.scrollHeight);
}

document.onload += onLoad();
