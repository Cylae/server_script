use criterion::{criterion_group, criterion_main, Criterion};
use server_manager::core::config::Config;
use server_manager::core::hardware::{HardwareInfo, HardwareProfile};
use server_manager::core::secrets::Secrets;
use server_manager::core::validate::{validate_safe_path, validate_service_name};
use server_manager::generate_compose_yaml;
use server_manager::services::{generate_port_matrix_markdown, get_all_services};

fn mock_hardware() -> HardwareInfo {
    HardwareInfo {
        profile: HardwareProfile::Standard,
        ram_gb: 8,
        cpu_cores: 4,
        has_nvidia: false,
        has_intel_quicksync: false,
        disk_gb: 512,
        swap_gb: 2,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    }
}

fn mock_secrets() -> Secrets {
    Secrets {
        mysql_root_password: Some("bench_root_secret".to_string()),
        mysql_user_password: Some("bench_user_secret".to_string()),
        nextcloud_db_password: Some("bench_nc_db".to_string()),
        glpi_db_password: Some("bench_glpi_db".to_string()),
        gitea_db_password: Some("bench_gitea_db".to_string()),
        yourls_admin_password: Some("bench_yourls_admin".to_string()),
        mailserver_password: Some("bench_mail_secret".to_string()),
        nextcloud_admin_password: Some("bench_nc_admin".to_string()),
        roundcube_db_password: Some("bench_rc_db".to_string()),
        vaultwarden_admin_token: Some("bench_vw_token".to_string()),
        server_manager_admin_password: Some("bench_sm_pass".to_string()),
    }
}

fn benchmark_catalog_retrieval(c: &mut Criterion) {
    c.bench_function("catalog_get_all_services", |b| {
        b.iter(|| {
            let services = get_all_services();
            criterion::black_box(services);
        })
    });
}

fn benchmark_compose_generation(c: &mut Criterion) {
    let hw = mock_hardware();
    let secrets = mock_secrets();
    let config = Config::default();

    c.bench_function("compose_generation_28_services", |b| {
        b.iter(|| {
            let yaml = generate_compose_yaml(
                criterion::black_box(&hw),
                criterion::black_box(&secrets),
                criterion::black_box(&config),
            );
            criterion::black_box(yaml)
        })
    });
}

fn benchmark_validation_throughput(c: &mut Criterion) {
    let service_names = [
        "sonarr",
        "radarr",
        "plex",
        "mariadb",
        "nextcloud",
        "vaultwarden",
        "nginx-proxy",
    ];
    let safe_paths = [
        "./config/plex",
        "./media/tv",
        "/var/run/docker.sock",
        "./config/mariadb/initdb.d",
    ];

    c.bench_function("validate_service_names", |b| {
        b.iter(|| {
            for name in &service_names {
                let res = validate_service_name(criterion::black_box(name));
                let _ = criterion::black_box(res);
            }
        })
    });

    c.bench_function("validate_safe_paths", |b| {
        b.iter(|| {
            for path in &safe_paths {
                let res = validate_safe_path(criterion::black_box(path));
                let _ = criterion::black_box(res);
            }
        })
    });
}

fn benchmark_port_matrix_generation(c: &mut Criterion) {
    c.bench_function("port_matrix_markdown_generation", |b| {
        b.iter(|| {
            let markdown = generate_port_matrix_markdown();
            criterion::black_box(markdown);
        })
    });
}

criterion_group!(
    benches,
    benchmark_catalog_retrieval,
    benchmark_compose_generation,
    benchmark_validation_throughput,
    benchmark_port_matrix_generation
);
criterion_main!(benches);
