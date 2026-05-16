use std::collections::BTreeMap;

use super::django::DjangoRequest;
use super::{django_assets, django_python};

pub(super) fn files(request: &DjangoRequest) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    files.insert(
        "requirements.txt".to_string(),
        "Django>=5.0,<6.0\n".to_string(),
    );
    files.insert(
        "manage.py".to_string(),
        django_python::manage_py(&request.project),
    );
    files.insert(format!("{}/__init__.py", request.project), String::new());
    files.insert(
        format!("{}/settings.py", request.project),
        django_python::settings_py(request),
    );
    files.insert(
        format!("{}/urls.py", request.project),
        django_python::project_urls_py(request),
    );
    files.insert(format!("{}/__init__.py", request.app), String::new());
    files.insert(
        format!("{}/apps.py", request.app),
        django_python::apps_py(&request.app),
    );
    files.insert(
        format!("{}/views.py", request.app),
        django_python::views_py(),
    );
    files.insert(
        format!("{}/urls.py", request.app),
        django_python::app_urls_py(),
    );
    files.insert(
        "templates/web/home.html".to_string(),
        django_assets::home_html(),
    );
    files.insert("static/web/site.css".to_string(), django_assets::site_css());
    files.insert(
        "docs/django-quickstart.md".to_string(),
        django_assets::quickstart_md(),
    );
    files
}
