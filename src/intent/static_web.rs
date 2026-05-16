use std::path::Path;

use super::{IntentOptions, IntentPreview};

pub(super) fn should_use(root: &Path, request: &str) -> bool {
    !root.join("package.json").exists() && is_web_app_request(request)
}

pub(super) fn preview(request: &str, options: &IntentOptions) -> IntentPreview {
    let adapter = options
        .agent_adapter
        .as_deref()
        .unwrap_or(crate::product_cli::config::DEFAULT_PROVIDER)
        .to_string();
    let mut defaults = super::defaults::resolve();
    defaults.agent_adapter = adapter.clone();
    IntentPreview {
        request: request.to_string(),
        inferred_intent: "code.static_web_app".to_string(),
        unknowns: Vec::new(),
        questions: Vec::new(),
        defaults,
        approval_required: options.approval_required,
        agent_spec_yaml: spec_yaml(&adapter, options.approval_required),
    }
}

fn is_web_app_request(request: &str) -> bool {
    let lower = request.to_lowercase();
    has_any(
        &lower,
        &[
            "web",
            "website",
            "app",
            "application",
            "сайт",
            "веб",
            "вэб",
            "прилож",
        ],
    ) && has_any(
        &lower,
        &["create", "build", "make", "созд", "сдел", "напиш"],
    )
}

fn spec_yaml(adapter: &str, approval_required: bool) -> String {
    let approval = if approval_required {
        "  approval_required: true\n"
    } else {
        ""
    };
    let execution = if adapter == "command" {
        format!(
            "  commands:\n    - |\n{}\n",
            indent(DEFAULT_INDEX_COMMAND, 6)
        )
    } else {
        "  commands: []\n".to_string()
    };
    format!(
        r#"task:
  id: create_static_web_app
  type: code.static_web_app
  title: Create animated static web app
  target: index.html

agent:
  adapter: {adapter}
  role: executor

workspace:
  type: code.git
  isolation: git_worktree

skills:
  - core.file.create

execution:
{execution}scope:
  allow:
    - index.html
    - assets/**
  deny:
    - .agent/**
    - .env*

rules:
  - R_SCOPE_ONLY

verify:
  profile: static_web
  commands:
    - test -f index.html

transaction:
{approval}  max_repair_attempts: 1
  rollback_on_failure: true
  commit_on_success: true
  memory_promotion: on_success
"#
    )
}

fn has_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn indent(value: &str, spaces: usize) -> String {
    let prefix = " ".repeat(spaces);
    value
        .lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

const DEFAULT_INDEX_COMMAND: &str = r#"cat > index.html <<'HTML'
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Motion Desk</title>
  <style>
    :root { color-scheme: dark; --bg: #111315; --panel: #1b2024; --line: #303941; --text: #f4f0e8; --muted: #aeb8b7; --green: #45c486; --gold: #e4b456; --coral: #f07064; }
    * { box-sizing: border-box; }
    body { margin: 0; min-height: 100vh; font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; background: var(--bg); color: var(--text); }
    main { min-height: 100vh; display: grid; grid-template-rows: auto 1fr; }
    header { display: flex; align-items: center; justify-content: space-between; gap: 1rem; padding: 1.25rem clamp(1rem, 4vw, 3rem); border-bottom: 1px solid var(--line); background: #15191c; }
    h1 { margin: 0; font-size: clamp(1.6rem, 5vw, 3.8rem); line-height: 1; }
    button { border: 1px solid #57636b; background: #f4f0e8; color: #15191c; min-height: 2.75rem; padding: 0 .95rem; font: inherit; font-weight: 750; cursor: pointer; }
    button:active { transform: translateY(1px); }
    .stage { display: grid; grid-template-columns: minmax(0, 1.1fr) minmax(280px, .9fr); gap: clamp(1rem, 3vw, 2rem); padding: clamp(1rem, 4vw, 3rem); }
    .timeline, .control { border: 1px solid var(--line); background: var(--panel); }
    .timeline { position: relative; min-height: 520px; overflow: hidden; }
    .track { position: absolute; inset: 2rem; display: grid; grid-template-columns: repeat(3, 1fr); gap: 1rem; }
    .lane { border-left: 1px solid var(--line); padding-left: 1rem; }
    .lane h2 { margin: 0 0 1rem; font-size: .85rem; text-transform: uppercase; letter-spacing: .08em; color: var(--muted); }
    .ticket { min-height: 7rem; margin-bottom: 1rem; padding: 1rem; background: #242b30; border: 1px solid #3a454d; animation: rise .55s ease both; }
    .ticket strong { display: block; margin-bottom: .45rem; font-size: 1rem; }
    .ticket span { color: var(--muted); line-height: 1.45; }
    .signal { position: absolute; left: 0; width: 100%; height: 2px; background: linear-gradient(90deg, transparent, var(--green), var(--gold), transparent); animation: scan 4s linear infinite; }
    .control { display: grid; align-content: start; gap: 1rem; padding: 1.25rem; }
    .meter { display: grid; gap: .75rem; }
    .bar { height: .7rem; background: #111518; border: 1px solid var(--line); overflow: hidden; }
    .bar span { display: block; height: 100%; width: 62%; background: var(--green); animation: load 3s ease-in-out infinite alternate; }
    input { width: 100%; min-height: 2.75rem; border: 1px solid #57636b; background: #111518; color: var(--text); padding: 0 .8rem; font: inherit; }
    .stats { display: grid; grid-template-columns: repeat(3, 1fr); gap: .75rem; }
    .stat { border: 1px solid var(--line); padding: .9rem; background: #15191c; }
    .stat b { display: block; font-size: 1.6rem; }
    .stat span { color: var(--muted); font-size: .85rem; }
    @keyframes scan { from { transform: translateY(0); } to { transform: translateY(520px); } }
    @keyframes rise { from { opacity: 0; transform: translateY(14px); } to { opacity: 1; transform: translateY(0); } }
    @keyframes load { from { width: 38%; } to { width: 88%; } }
    @media (max-width: 760px) { header, .stage { padding: 1rem; } .stage { grid-template-columns: 1fr; } .timeline { min-height: 620px; } .track { grid-template-columns: 1fr; inset: 1rem; } .lane { min-height: 170px; } }
  </style>
</head>
<body>
  <main>
    <header>
      <h1>Motion Desk</h1>
      <button id="add">Add Task</button>
    </header>
    <section class="stage">
      <div class="timeline">
        <div class="signal"></div>
        <div class="track">
          <section class="lane" data-lane="0"><h2>Plan</h2></section>
          <section class="lane" data-lane="1"><h2>Build</h2></section>
          <section class="lane" data-lane="2"><h2>Launch</h2></section>
        </div>
      </div>
      <aside class="control">
        <input id="title" value="Animated interaction pass" aria-label="Task title">
        <button id="shuffle">Shuffle Flow</button>
        <div class="meter">
          <span>Momentum</span>
          <div class="bar"><span></span></div>
        </div>
        <div class="stats">
          <div class="stat"><b id="count">6</b><span>tasks</span></div>
          <div class="stat"><b>3</b><span>lanes</span></div>
          <div class="stat"><b>24h</b><span>cycle</span></div>
        </div>
      </aside>
    </section>
  </main>
  <script>
    const lanes = [...document.querySelectorAll('.lane')];
    const count = document.querySelector('#count');
    const title = document.querySelector('#title');
    const samples = ['Prototype motion', 'Tune spacing', 'Connect controls', 'Polish mobile', 'Review states', 'Ship preview'];
    let total = 0;
    function addTicket(text = samples[total % samples.length], lane = total % lanes.length) {
      const ticket = document.createElement('article');
      ticket.className = 'ticket';
      ticket.innerHTML = `<strong>${text}</strong><span>Responsive layout with visible motion and stable controls.</span>`;
      lanes[lane].append(ticket);
      total += 1;
      count.textContent = total;
    }
    samples.forEach((item, index) => addTicket(item, index % lanes.length));
    document.querySelector('#add').addEventListener('click', () => addTicket(title.value || 'New task'));
    document.querySelector('#shuffle').addEventListener('click', () => {
      document.querySelectorAll('.ticket').forEach((ticket, index) => lanes[(index + total) % lanes.length].append(ticket));
    });
  </script>
</body>
</html>
HTML"#;
