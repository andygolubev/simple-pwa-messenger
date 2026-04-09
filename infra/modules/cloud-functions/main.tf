resource "google_storage_bucket" "function_source" {
  name                        = "${var.project_id}-messenger-fn-src-${var.environment}"
  project                     = var.project_id
  location                    = var.region
  uniform_bucket_level_access = true

  lifecycle {
    prevent_destroy = true
  }
}

resource "google_storage_bucket_object" "archive" {
  name   = "${var.function_name}.zip"
  bucket = google_storage_bucket.function_source.name
  source = var.source_archive
}

resource "google_cloudfunctions2_function" "api" {
  name     = var.function_name
  project  = var.project_id
  location = var.region

  build_config {
    runtime     = var.runtime
    entry_point = var.entry_point

    source {
      storage_source {
        bucket = google_storage_bucket.function_source.name
        object = google_storage_bucket_object.archive.name
      }
    }
  }

  service_config {
    available_memory      = "${var.memory_mb}M"
    timeout_seconds       = var.timeout_seconds
    min_instance_count    = var.min_instance_count
    max_instance_count    = var.max_instance_count
    ingress_settings      = "ALLOW_ALL"
    service_account_email = var.service_account_email
    environment_variables = var.environment_variables
  }
}

resource "google_cloud_run_service_iam_member" "public_invoker" {
  count = var.allow_unauthenticated_invoker ? 1 : 0

  project  = var.project_id
  location = var.region
  service  = google_cloudfunctions2_function.api.service_config[0].service
  role     = "roles/run.invoker"
  member   = "allUsers"
}
