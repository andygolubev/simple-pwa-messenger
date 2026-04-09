locals {
  required_services = [
    "artifactregistry.googleapis.com",
    "cloudbuild.googleapis.com",
    "cloudfunctions.googleapis.com",
    "firestore.googleapis.com",
    "iam.googleapis.com",
    "identitytoolkit.googleapis.com",
    "logging.googleapis.com",
    "run.googleapis.com",
    "secretmanager.googleapis.com",
  ]
}

module "project_services" {
  source = "./modules/project-services"

  project_id = var.project_id
  services   = local.required_services
}

module "iam" {
  source = "./modules/iam"

  project_id                  = var.project_id
  environment                 = var.environment
  function_service_account_id = "messenger-fn-${var.environment}"
  depends_on                  = [module.project_services]
}

module "identity_platform" {
  source = "./modules/identity-platform"

  project_id                 = var.project_id
  autodelete_anonymous_users = var.identity_platform_autodelete_anonymous_users
  google_oauth_client_id     = var.google_oauth_client_id
  google_oauth_client_secret = var.google_oauth_client_secret
  depends_on                 = [module.project_services]
}

module "firestore" {
  source = "./modules/firestore"

  project_id  = var.project_id
  location_id = var.firestore_location
  depends_on  = [module.project_services]
}

module "secret_manager" {
  source = "./modules/secret-manager"

  project_id                     = var.project_id
  function_service_account_email = module.iam.function_service_account_email
  secret_names                   = var.secret_names
  secret_initial_values          = var.secret_initial_values
  secret_version_destroy_ttl     = var.secret_version_destroy_ttl
  depends_on                     = [module.project_services, module.iam]
}

module "cloud_functions" {
  source = "./modules/cloud-functions"

  project_id            = var.project_id
  region                = var.region
  environment           = var.environment
  function_name         = var.function_name
  runtime               = var.function_runtime
  entry_point           = var.function_entry_point
  source_archive        = var.function_source_archive
  service_account_email = module.iam.function_service_account_email
  memory_mb             = var.function_memory_mb
  timeout_seconds       = var.function_timeout_seconds
  min_instance_count    = var.function_min_instances
  max_instance_count    = var.function_max_instances
  environment_variables = merge(
    {
      ENVIRONMENT             = var.environment
      FIRESTORE_DATABASE      = module.firestore.database_name
      JWT_SECRET_ID           = module.secret_manager.secret_ids["jwt_signing_key"]
      VAPID_PRIVATE_SECRET_ID = module.secret_manager.secret_ids["vapid_private_key"]
      VAPID_PUBLIC_SECRET_ID  = module.secret_manager.secret_ids["vapid_public_key"]
    },
    var.function_environment_variables
  )
  depends_on = [
    module.project_services,
    module.iam,
    module.secret_manager,
    module.firestore
  ]
}
