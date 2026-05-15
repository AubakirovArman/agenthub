# AgentHub PRD Tracker

Original PRD `prd.md` ішінде қалады. Working tracker `prd/` ішінде.

Қолдану:

```bash
ls prd/todo
ls prd/done
sed -n '1,120p' prd/status.md
```

Ережелер:

- Phases сандық ретпен орындалады.
- Partial phases `prd/todo/` ішінде қалады.
- Code, tests, acceptance және 4-language docs толық біткенде ғана phase `prd/done/` ішіне көшеді.
- Closing commit hash phase file және `prd/status.md` ішіне жазылады.

Қазіргі келесі phase: жоқ; барлық tracked PRD phases done.
