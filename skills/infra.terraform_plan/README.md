# infra.terraform_plan

Validates Terraform and produces a plan artifact.

## Example AgentSpec

```yaml
skills: [infra.terraform_plan]
workspace:
  type: infra.git
verify:
  commands: ["terraform fmt -check", "terraform validate", "terraform plan -out=tfplan"]
```

Success test: plan is generated. Failure test: `terraform apply` is blocked.
