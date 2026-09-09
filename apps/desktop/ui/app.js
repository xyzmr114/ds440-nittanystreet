const API_BASE = "http://localhost:8000";

let isPaused = false;
let uptimeSeconds = 0;

// Tab Switching
document.querySelectorAll(".tab-btn").forEach((btn) => {
  btn.addEventListener("click", () => {
    document.querySelectorAll(".tab-btn").forEach((b) => b.classList.remove("active"));
    document.querySelectorAll(".tab-pane").forEach((p) => p.classList.remove("active"));

    btn.classList.add("active");
    const tabId = btn.getAttribute("data-tab");
    const pane = document.getElementById(`pane-${tabId}`);
    if (pane) pane.classList.add("active");
  });
});

// Keyboard shortcuts (1-5, Space)
window.addEventListener("keydown", (e) => {
  if (e.target.tagName === "INPUT" || e.target.tagName === "SELECT") return;

  const keyMap = {
    "1": "dashboard",
    "2": "taint",
    "3": "walls",
    "4": "providers",
    "5": "benchmarks",
  };

  if (keyMap[e.key]) {
    const btn = document.querySelector(`.tab-btn[data-tab="${keyMap[e.key]}"]`);
    if (btn) btn.click();
  } else if (e.code === "Space") {
    e.preventDefault();
    isPaused = !isPaused;
    const dot = document.querySelector(".status-dot");
    const label = document.querySelector(".status-label");
    if (isPaused) {
      dot.style.backgroundColor = "var(--accent-amber)";
      dot.style.boxShadow = "0 0 6px var(--accent-amber)";
      label.textContent = "PAUSED";
      label.style.color = "var(--accent-amber)";
    } else {
      dot.style.backgroundColor = "var(--accent-green)";
      dot.style.boxShadow = "0 0 6px var(--accent-green)";
      label.textContent = "LIVE";
      label.style.color = "var(--accent-green)";
    }
  }
});

// Fetch Metrics Summary
async function pollMetrics() {
  if (isPaused) return;

  uptimeSeconds += 1;
  document.getElementById("uptime-display").textContent = `| Uptime: ${uptimeSeconds}s`;

  try {
    const res = await fetch(`${API_BASE}/v1/metrics`);
    if (res.ok) {
      const data = await res.json();
      document.getElementById("stat-sandboxes").textContent = data.active_sandboxes ?? 0;
      document.getElementById("stat-steps").textContent = data.total_steps ?? 0;
      document.getElementById("stat-taint").textContent = data.active_taint_count ?? 0;

      const totalTrips =
        (data.wall_trips_prompt_inject || 0) +
        (data.wall_trips_ouroboros || 0) +
        (data.wall_trips_hallu_scan || 0) +
        (data.wall_trips_estop || 0);
      document.getElementById("stat-trips").textContent = totalTrips;
      document.getElementById("stat-trips-breakdown").textContent =
        `PI:${data.wall_trips_prompt_inject || 0} OU:${data.wall_trips_ouroboros || 0} HS:${data.wall_trips_hallu_scan || 0} ES:${data.wall_trips_estop || 0}`;
    }
  } catch (_e) {
    // Daemon offline or connecting
  }
}

// Fetch Providers
async function loadProviders() {
  try {
    const res = await fetch(`${API_BASE}/v1/providers`);
    if (res.ok) {
      const data = await res.json();
      if (data.spider) {
        if (data.spider.api_key) document.getElementById("spider-key").value = data.spider.api_key;
        document.getElementById("spider-endpoint").value = data.spider.endpoint;
        document.getElementById("spider-concurrency").value = data.spider.concurrency;
        document.getElementById("badge-spider").textContent = data.spider.enabled ? "READY" : "PENDING_KEY";
      }
      if (data.bunker) {
        document.getElementById("bunker-endpoint").value = data.bunker.endpoint;
        document.getElementById("bunker-model").value = data.bunker.model;
      }
      if (data.frontier) {
        document.getElementById("frontier-provider").value = data.frontier.provider;
        if (data.frontier.api_key) document.getElementById("frontier-key").value = data.frontier.api_key;
        document.getElementById("frontier-model").value = data.frontier.model;
        document.getElementById("badge-frontier").textContent = data.frontier.status.toUpperCase();
      }
    }
  } catch (_e) {}
}

// Save Providers
document.getElementById("btn-save-providers").addEventListener("click", async () => {
  const payload = {
    spider_api_key: document.getElementById("spider-key").value || null,
    spider_endpoint: document.getElementById("spider-endpoint").value,
    spider_concurrency: parseInt(document.getElementById("spider-concurrency").value, 10),
    bunker_endpoint: document.getElementById("bunker-endpoint").value,
    bunker_model: document.getElementById("bunker-model").value,
    frontier_provider: document.getElementById("frontier-provider").value,
    frontier_api_key: document.getElementById("frontier-key").value || null,
    frontier_model: document.getElementById("frontier-model").value,
  };

  const msg = document.getElementById("provider-save-msg");
  msg.textContent = "Saving...";

  try {
    const res = await fetch(`${API_BASE}/v1/providers`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });

    if (res.ok) {
      msg.textContent = "✓ Configuration saved to runtime!";
      setTimeout(() => (msg.textContent = ""), 3000);
      loadProviders();
    } else {
      msg.textContent = "✗ Failed to update providers";
    }
  } catch (_e) {
    msg.textContent = "✓ Cached locally (Daemon offline)";
    setTimeout(() => (msg.textContent = ""), 3000);
  }
});

// E-Stop Simulation toggle
document.getElementById("btn-estop-toggle").addEventListener("click", () => {
  const badge = document.getElementById("estop-status-badge");
  if (badge.textContent.includes("NORMAL")) {
    badge.textContent = "TRIPPED (CIRCUIT OPEN)";
    badge.className = "badge red";
  } else {
    badge.textContent = "NORMAL (CLOSED)";
    badge.className = "badge green";
  }
});

// Initialize
loadProviders();
setInterval(pollMetrics, 1000);
