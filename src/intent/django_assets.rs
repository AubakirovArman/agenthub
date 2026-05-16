pub(super) fn home_html() -> String {
    r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>AgentHub Django</title>
    <link rel="stylesheet" href="/static/web/site.css">
  </head>
  <body>
    <main>
      <p class="eyebrow">AgentHub scaffold</p>
      <h1>Django web application</h1>
      <p>Created through an AgentHub transaction with scoped files and verifier checks.</p>
    </main>
  </body>
</html>
"#
    .to_string()
}

pub(super) fn site_css() -> String {
    r#"body {
  margin: 0;
  min-height: 100vh;
  display: grid;
  place-items: center;
  color: #172026;
  background: #f4f7f8;
  font-family: Arial, Helvetica, sans-serif;
}

main {
  width: min(720px, calc(100% - 48px));
}

.eyebrow {
  color: #3b6f77;
  font-size: 0.8rem;
  font-weight: 700;
  letter-spacing: 0;
  text-transform: uppercase;
}

h1 {
  margin: 0 0 12px;
  font-size: clamp(2rem, 8vw, 4rem);
  line-height: 1;
}

p {
  max-width: 42rem;
  line-height: 1.6;
}
"#
    .to_string()
}

pub(super) fn quickstart_md() -> String {
    r#"# Django Quickstart

This project was scaffolded by AgentHub.

```bash
python -m venv .venv
. .venv/bin/activate
pip install -r requirements.txt
python manage.py migrate
python manage.py runserver
```

Open http://127.0.0.1:8000 after the development server starts.
"#
    .to_string()
}
