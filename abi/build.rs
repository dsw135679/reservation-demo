use std::process::Command;

use tonic_build::Builder;

trait BuilderExt {
    /// 为 struct 设置 sqlx::Type
    fn with_sql_type(self, paths: &[&str]) -> Self;
    /// 为 struct 设置 derive_builder
    fn with_builder(self, paths: &[&str]) -> Self;
    /// 为 fields 设置 into
    fn with_builder_into(self, path: &str, fields: &[&str]) -> Self;
    /// 为 fields 设置 option
    fn with_builder_option(self, path: &str, fields: &[&str]) -> Self;
}

impl BuilderExt for Builder {
    fn with_sql_type(self, paths: &[&str]) -> Self {
        paths.iter().fold(self, |acc, path| {
            acc.type_attribute(path, "#[derive(sqlx::Type)]")
        })
    }

    fn with_builder(self, paths: &[&str]) -> Self {
        paths.iter().fold(self, |acc, path| {
            acc.type_attribute(path, "#[derive(derive_builder::Builder)]")
        })
    }

    fn with_builder_into(self, path: &str, fields: &[&str]) -> Self {
        fields.iter().fold(self, |acc, field| {
            acc.field_attribute(
                &format!("{}.{}", path, field),
                "#[builder(setter(into),default)]",
            )
        })
    }

    fn with_builder_option(self, path: &str, fields: &[&str]) -> Self {
        fields.iter().fold(self, |acc, field| {
            acc.field_attribute(
                &format!("{}.{}", path, field),
                "#[builder(setter(into,strip_option))]",
            )
        })
    }
}

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        .with_sql_type(&["reservation.ReservationStatus"])
        .with_builder(&["reservation.ReservationQuery"])
        .with_builder_into(
            "reservation.ReservationQuery",
            &["resource_id", "user_id", "status", "page", "size", "desc"],
        )
        .with_builder_option("reservation.ReservationQuery", &["start", "end"])
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();

    //fs::remove_file("src/pb/google.protobuf.rs").unwrap();

    Command::new("cargo").args(&["fmt"]).output().unwrap();

    println!("cargo:rerun-if-changed=protos/reservation.proto");
}
