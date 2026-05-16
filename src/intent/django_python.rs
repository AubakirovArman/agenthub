use super::django::DjangoRequest;

pub(super) fn manage_py(project: &str) -> String {
    format!(
        r#"#!/usr/bin/env python
import os
import sys


def main():
    os.environ.setdefault("DJANGO_SETTINGS_MODULE", "{project}.settings")
    from django.core.management import execute_from_command_line

    execute_from_command_line(sys.argv)


if __name__ == "__main__":
    main()
"#
    )
}

pub(super) fn settings_py(request: &DjangoRequest) -> String {
    format!(
        r#"from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
SECRET_KEY = "agenthub-local-dev"
DEBUG = True
ALLOWED_HOSTS = []

INSTALLED_APPS = [
    "django.contrib.admin",
    "django.contrib.auth",
    "django.contrib.contenttypes",
    "django.contrib.sessions",
    "django.contrib.messages",
    "django.contrib.staticfiles",
    "{app}",
]

MIDDLEWARE = [
    "django.middleware.security.SecurityMiddleware",
    "django.contrib.sessions.middleware.SessionMiddleware",
    "django.middleware.common.CommonMiddleware",
    "django.middleware.csrf.CsrfViewMiddleware",
    "django.contrib.auth.middleware.AuthenticationMiddleware",
    "django.contrib.messages.middleware.MessageMiddleware",
    "django.middleware.clickjacking.XFrameOptionsMiddleware",
]

ROOT_URLCONF = "{project}.urls"

TEMPLATES = [
    {{
        "BACKEND": "django.template.backends.django.DjangoTemplates",
        "DIRS": [BASE_DIR / "templates"],
        "APP_DIRS": True,
        "OPTIONS": {{
            "context_processors": [
                "django.template.context_processors.debug",
                "django.template.context_processors.request",
                "django.contrib.auth.context_processors.auth",
                "django.contrib.messages.context_processors.messages",
            ],
        }},
    }},
]

DATABASES = {{
    "default": {{
        "ENGINE": "django.db.backends.sqlite3",
        "NAME": BASE_DIR / "db.sqlite3",
    }}
}}

LANGUAGE_CODE = "en-us"
TIME_ZONE = "UTC"
USE_I18N = True
USE_TZ = True
STATIC_URL = "static/"
DEFAULT_AUTO_FIELD = "django.db.models.BigAutoField"
"#,
        app = request.app,
        project = request.project
    )
}

pub(super) fn project_urls_py(request: &DjangoRequest) -> String {
    format!(
        r#"from django.contrib import admin
from django.urls import include, path

urlpatterns = [
    path("admin/", admin.site.urls),
    path("", include("{app}.urls")),
]
"#,
        app = request.app
    )
}

pub(super) fn apps_py(app: &str) -> String {
    format!(
        r#"from django.apps import AppConfig


class WebConfig(AppConfig):
    default_auto_field = "django.db.models.BigAutoField"
    name = "{app}"
"#
    )
}

pub(super) fn views_py() -> String {
    r#"from django.shortcuts import render


def home(request):
    return render(request, "web/home.html", {"title": "AgentHub Django"})
"#
    .to_string()
}

pub(super) fn app_urls_py() -> String {
    r#"from django.urls import path

from . import views

urlpatterns = [
    path("", views.home, name="home"),
]
"#
    .to_string()
}
