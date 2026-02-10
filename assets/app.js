// forja monitor — dashboard client
(function () {
    "use strict";

    const $ = (sel) => document.querySelector(sel);

    // State
    let teams = {};
    let tasksByTeam = {};
    let messagesByKey = {};

    // Elements
    const statusDot = $("#status-dot");
    const statusText = $("#status-text");
    const teamsList = $("#teams-list");
    const tasksPending = $("#tasks-pending");
    const tasksInProgress = $("#tasks-in-progress");
    const tasksCompleted = $("#tasks-completed");
    const messagesFeed = $("#messages-feed");
    const activityLog = $("#activity-log");

    function connect() {
        const es = new EventSource("/api/events");

        es.onopen = () => {
            statusDot.className = "dot connected";
            statusText.textContent = "Connected";
        };

        es.onerror = () => {
            statusDot.className = "dot disconnected";
            statusText.textContent = "Reconnecting...";
        };

        es.onmessage = (e) => {
            try {
                const event = JSON.parse(e.data);
                handleEvent(event);
            } catch (err) {
                console.error("Failed to parse event:", err);
            }
        };
    }

    function handleEvent(event) {
        switch (event.type) {
            case "Snapshot":
                handleSnapshot(event);
                break;
            case "TeamUpdated":
                teams[event.team.name] = event.team;
                renderTeams();
                logActivity("Team updated: " + event.team.name);
                break;
            case "TeamDeleted":
                delete teams[event.team_name];
                delete tasksByTeam[event.team_name];
                renderTeams();
                renderTasks();
                logActivity("Team removed: " + event.team_name);
                break;
            case "TaskUpdated":
                if (!tasksByTeam[event.team_name]) tasksByTeam[event.team_name] = {};
                tasksByTeam[event.team_name][event.task.id] = event.task;
                renderTasks();
                logActivity(
                    "Task #" + event.task.id + " \u2192 " + event.task.status +
                    (event.task.owner ? " (" + event.task.owner + ")" : "")
                );
                break;
            case "TaskDeleted":
                if (tasksByTeam[event.team_name]) {
                    delete tasksByTeam[event.team_name][event.task_id];
                }
                renderTasks();
                logActivity("Task deleted: #" + event.task_id);
                break;
            case "MessageReceived":
                var key = event.team_name + "/" + event.recipient;
                if (!messagesByKey[key]) messagesByKey[key] = [];
                messagesByKey[key].push(event.message);
                renderMessages();
                logActivity(event.message.from + " \u2192 " + event.recipient);
                break;
            case "Heartbeat":
                break;
        }
    }

    function handleSnapshot(snapshot) {
        teams = {};
        (snapshot.teams || []).forEach(function (t) { teams[t.name] = t; });

        tasksByTeam = {};
        (snapshot.tasks || []).forEach(function (group) {
            tasksByTeam[group.team_name] = {};
            (group.tasks || []).forEach(function (task) {
                tasksByTeam[group.team_name][task.id] = task;
            });
        });

        messagesByKey = {};
        (snapshot.messages || []).forEach(function (group) {
            var key = group.team_name + "/" + group.recipient;
            messagesByKey[key] = group.messages || [];
        });

        renderTeams();
        renderTasks();
        renderMessages();
        logActivity("Dashboard connected");
    }

    // --- Safe DOM builders (no innerHTML with untrusted content) ---

    function el(tag, className, textContent) {
        var node = document.createElement(tag);
        if (className) node.className = className;
        if (textContent) node.textContent = textContent;
        return node;
    }

    function renderTeams() {
        var names = Object.keys(teams);
        teamsList.textContent = "";

        if (names.length === 0) {
            teamsList.appendChild(el("p", "empty", "No active teams"));
            return;
        }

        names.forEach(function (name) {
            var team = teams[name];
            var card = el("div", "team-card");

            card.appendChild(el("div", "team-name", team.name));
            if (team.description) {
                card.appendChild(el("div", "team-desc", team.description));
            }

            (team.members || []).forEach(function (m) {
                var member = el("div", "member");
                var dot = el("span", "member-dot color-" + (m.color || "default"));
                member.appendChild(dot);
                member.appendChild(el("span", "member-name", m.name));
                member.appendChild(el("span", "member-type", m.agent_type));
                card.appendChild(member);
            });

            teamsList.appendChild(card);
        });
    }

    function renderTasks() {
        var pending = [];
        var inProgress = [];
        var completed = [];

        Object.keys(tasksByTeam).forEach(function (teamName) {
            var tasksMap = tasksByTeam[teamName];
            Object.keys(tasksMap).forEach(function (id) {
                var task = tasksMap[id];
                var card = buildTaskCard(task, teamName);
                switch (task.status) {
                    case "pending": pending.push(card); break;
                    case "in_progress": inProgress.push(card); break;
                    case "completed": completed.push(card); break;
                }
            });
        });

        fillContainer(tasksPending, pending, "None");
        fillContainer(tasksInProgress, inProgress, "None");
        fillContainer(tasksCompleted, completed, "None");
    }

    function buildTaskCard(task, teamName) {
        var blocked = task.blocked_by && task.blocked_by.length > 0;
        var card = el("div", "task-card" + (blocked ? " task-blocked" : ""));

        card.appendChild(el("div", "task-subject", task.subject));

        if (task.status === "in_progress" && task.active_form) {
            var af = el("div", null, task.active_form);
            af.style.color = "var(--yellow)";
            af.style.fontSize = "10px";
            af.style.marginTop = "2px";
            card.appendChild(af);
        }

        var meta = el("div", "task-meta");
        meta.appendChild(el("span", "task-owner", task.owner || "unassigned"));
        meta.appendChild(el("span", "task-id", "#" + task.id + " \u00b7 " + teamName));
        card.appendChild(meta);

        return card;
    }

    function renderMessages() {
        var allMessages = [];

        Object.keys(messagesByKey).forEach(function (key) {
            var parts = key.split("/");
            var teamName = parts[0];
            var recipient = parts.slice(1).join("/");
            messagesByKey[key].forEach(function (msg) {
                allMessages.push({
                    from: msg.from,
                    text: msg.text,
                    timestamp: msg.timestamp,
                    color: msg.color,
                    read: msg.read,
                    recipient: recipient,
                    teamName: teamName
                });
            });
        });

        allMessages.sort(function (a, b) {
            return (a.timestamp || "").localeCompare(b.timestamp || "");
        });

        messagesFeed.textContent = "";

        if (allMessages.length === 0) {
            messagesFeed.appendChild(el("p", "empty", "No messages yet"));
            return;
        }

        var recent = allMessages.slice(-50);

        recent.forEach(function (msg) {
            var item = el("div", "message-item");
            item.style.borderLeftColor = "var(--" + (msg.color || "text-dim") + ")";

            var header = el("div", "message-header");
            var left = el("span");
            left.appendChild(el("span", "message-from", msg.from));
            var toSpan = el("span", "message-to", " \u2192 " + msg.recipient);
            left.appendChild(toSpan);
            header.appendChild(left);
            header.appendChild(el("span", "message-time", formatTime(msg.timestamp)));
            item.appendChild(header);

            var displayText = extractMessageText(msg.text);
            item.appendChild(el("div", "message-text", displayText));

            messagesFeed.appendChild(item);
        });

        messagesFeed.scrollTop = messagesFeed.scrollHeight;
    }

    function extractMessageText(text) {
        try {
            var parsed = JSON.parse(text);
            if (parsed.content) return parsed.content;
            if (parsed.subject) return parsed.type + ": " + parsed.subject;
            if (parsed.text) return parsed.text;
        } catch (e) {
            // Not JSON — use raw text
        }
        if (text.length > 300) return text.slice(0, 300) + "...";
        return text;
    }

    function logActivity(text) {
        var empty = activityLog.querySelector(".empty");
        if (empty) empty.remove();

        var item = el("div", "activity-item");
        item.appendChild(el("span", "activity-time", formatTime(new Date().toISOString())));
        item.appendChild(el("span", "activity-text", text));
        activityLog.appendChild(item);

        while (activityLog.children.length > 100) {
            activityLog.removeChild(activityLog.firstChild);
        }

        activityLog.scrollTop = activityLog.scrollHeight;
    }

    function fillContainer(container, elements, emptyText) {
        container.textContent = "";
        if (elements.length === 0) {
            container.appendChild(el("p", "empty", emptyText));
        } else {
            elements.forEach(function (node) { container.appendChild(node); });
        }
    }

    function formatTime(isoString) {
        if (!isoString) return "";
        try {
            var d = new Date(isoString);
            return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
        } catch (e) {
            return isoString;
        }
    }

    // Start
    connect();
})();
