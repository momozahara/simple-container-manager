const labelStatus = document.getElementById("status");
const logsContainer = document.getElementById("logs-container");
const textLogs = document.getElementById("logs");
const buttonStart = document.getElementById("start");
const buttonStop = document.getElementById("stop");
const checkStick = document.getElementById("fn-1");
let interval;
let intervalLogs;
let stickToBottom = true;

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
  return fetch("/api/stream",
    {
      method: "POST",
      keepalive: true,
      cache: "no-cache",
    })
    .then((response) => {
      const reader = response.body.getReader();
      function read() {
        return reader.read().then(({ done, value }) => {
          if (done) {
            return;
          }
          const decoder = new TextDecoder();
          textLogs.innerText += decoder.decode(value);

          if (stickToBottom) {
            logsContainer.scrollTo(0, logsContainer.scrollHeight);
          }

          return read();
        });
      }
      return read();
    })
    .catch((e) => {
      console.error(e);
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

function onStickToggle() {
  console.log("Change");
  if (checkStick.checked) {
    stickToBottom = true;
  } else {
    stickToBottom = false;
  }
}

async function onLoad() {
  getStatus();
  interval = setInterval(getStatus, 6000);
  await getLogs();
  intervalLogs = setInterval(getLogs, 10000);
}

document.onload += onLoad();
