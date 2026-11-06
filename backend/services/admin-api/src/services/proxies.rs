//! Generated thin-BFF proxies — a table of `passthrough_proxy!`
//! invocations, one per BFF face (see services/proxy_kit.rs for the
//! pass/drop/stub/hand method kinds; the BFF and domain methods share
//! their message types by contract, so no mapping rides here).
//! DO NOT EDIT; regenerate via scripts/gen-proxies.py when the
//! contract re-syncs. Hand-written faces (authentication — captcha/
//! cookie concerns; admin-portal aggregation; file-transfer multipart)
//! live in their own modules.
#![allow(clippy::all)]
#![allow(missing_docs)]

use crate::passthrough_proxy;
use proto::proto::access_key::service::v1 as access_key_v1;
use proto::proto::ai::service::v1 as ai_v1;
use proto::proto::audit::service::v1 as audit_v1;
use proto::proto::authentication::service::v1 as authentication_v1;
use proto::proto::comment::service::v1 as comment_v1;
use proto::proto::config::service::v1 as config_v1;
use proto::proto::content::service::v1 as content_v1;
use proto::proto::dict::service::v1 as dict_v1;
use proto::proto::flow::service::v1 as flow_v1;
use proto::proto::identity::service::v1 as identity_v1;
use proto::proto::interaction::service::v1 as interaction_v1;
use proto::proto::internal_message::service::v1 as internal_message_v1;
use proto::proto::media::service::v1 as media_v1;
use proto::proto::monitor_alert::service::v1 as monitor_alert_v1;
use proto::proto::notify::service::v1 as notify_v1;
use proto::proto::online_session::service::v1 as online_session_v1;
use proto::proto::pagination;
use proto::proto::permission::service::v1 as permission_v1;
use proto::proto::redis_cache::service::v1 as redis_cache_v1;
use proto::proto::server_monitor::service::v1 as server_monitor_v1;
use proto::proto::site::service::v1 as site_v1;
use proto::proto::social::service::v1 as social_v1;
use proto::proto::stats::service::v1 as stats_v1;
use proto::proto::storage::service::v1 as storage_v1;
use proto::proto::task::service::v1 as task_v1;
use proto::proto::translator::service::v1 as translator_v1;
use proto::proto::webhook::service::v1 as webhook_v1;

passthrough_proxy! {
    /// The pass-through proxy of `AccessKeyServiceHandlers`.
    proto::gen_admin::services::AccessKeyServiceHandlers for AccessKeyProxy {
        client: access_key_v1::access_key_service_client::AccessKeyServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> access_key_v1::ListAccessKeyResponse,
            pass get(access_key_v1::GetAccessKeyRequest) -> access_key_v1::AccessKey,
            pass create(access_key_v1::CreateAccessKeyRequest)
                -> access_key_v1::CreateAccessKeyResponse,
            pass update(access_key_v1::UpdateAccessKeyRequest) -> pbjson_types::Empty,
            pass delete(access_key_v1::DeleteAccessKeyRequest) -> pbjson_types::Empty,
            pass reset_secret(access_key_v1::ResetAccessKeySecretRequest)
                -> access_key_v1::CreateAccessKeyResponse,
            pass issue_token(access_key_v1::IssueTokenRequest) -> access_key_v1::IssueTokenResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `AiAgentServiceHandlers`.
    proto::gen_admin::services::AiAgentServiceHandlers for AiAgentProxy {
        client: ai_v1::ai_agent_service_client::AiAgentServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> ai_v1::ListAiAgentAppsResponse,
            pass get(ai_v1::GetAiAgentAppRequest) -> ai_v1::AiAgentApp,
            pass create(ai_v1::CreateAiAgentAppRequest) -> ai_v1::AiAgentApp,
            pass update(ai_v1::UpdateAiAgentAppRequest) -> pbjson_types::Empty,
            pass delete(ai_v1::DeleteAiAgentAppRequest) -> pbjson_types::Empty,
            pass approve_run(ai_v1::ApproveAgentRunRequest) -> ai_v1::AiAgentRun,
            pass run(ai_v1::RunAiAgentAppRequest) -> ai_v1::AiAgentRun,
            pass list_runs(pagination::PagingRequest) -> ai_v1::ListAiAgentRunsResponse,
            pass get_run(ai_v1::GetAiAgentRunRequest) -> ai_v1::AiAgentRun,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `AiChatServiceHandlers`.
    proto::gen_admin::services::AiChatServiceHandlers for AiChatProxy {
        client: ai_v1::ai_chat_service_client::AiChatServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            stub chat(ai_v1::ChatRequest) -> ai_v1::ChatResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `AiContentServiceHandlers`.
    proto::gen_admin::services::AiContentServiceHandlers for AiContentProxy {
        client: ai_v1::ai_content_service_client::AiContentServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass generate_content(ai_v1::GenerateContentRequest) -> ai_v1::GenerateContentResponse,
            pass semantic_search(ai_v1::SemanticSearchRequest) -> ai_v1::SemanticSearchResponse,
            pass rebuild_search_index(pbjson_types::Empty) -> ai_v1::RebuildSearchIndexResponse,
            pass get_safety_insights(ai_v1::SafetyInsightsRequest) -> ai_v1::SafetyInsightsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `AiConversationServiceHandlers`.
    proto::gen_admin::services::AiConversationServiceHandlers for AiConversationProxy {
        client: ai_v1::ai_conversation_service_client::AiConversationServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> ai_v1::ListAiConversationResponse,
            pass get(ai_v1::GetAiConversationRequest) -> ai_v1::AiConversation,
            pass update(ai_v1::UpdateAiConversationRequest) -> pbjson_types::Empty,
            pass delete(ai_v1::DeleteAiConversationRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `AiKnowledgeBaseServiceHandlers`.
    proto::gen_admin::services::AiKnowledgeBaseServiceHandlers for AiKnowledgeBaseProxy {
        client: ai_v1::ai_knowledge_base_service_client::AiKnowledgeBaseServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> ai_v1::ListAiKnowledgeBaseResponse,
            pass get(ai_v1::GetAiKnowledgeBaseRequest) -> ai_v1::AiKnowledgeBase,
            pass create(ai_v1::CreateAiKnowledgeBaseRequest) -> pbjson_types::Empty,
            pass update(ai_v1::UpdateAiKnowledgeBaseRequest) -> pbjson_types::Empty,
            pass delete(ai_v1::DeleteAiKnowledgeBaseRequest) -> pbjson_types::Empty,
            pass upload_doc(ai_v1::UploadAiDocRequest) -> ai_v1::UploadAiDocResponse,
            pass upload_doc_file(ai_v1::UploadAiDocFileRequest) -> ai_v1::UploadAiDocResponse,
            pass list_docs(ai_v1::ListAiDocsRequest) -> ai_v1::ListAiDocsResponse,
            pass delete_doc(ai_v1::DeleteAiDocRequest) -> pbjson_types::Empty,
            pass search(ai_v1::SearchAiKnowledgeRequest) -> ai_v1::SearchAiKnowledgeResponse,
            pass reindex_docs(ai_v1::ReindexDocsRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `AiMessageServiceHandlers`.
    proto::gen_admin::services::AiMessageServiceHandlers for AiMessageProxy {
        client: ai_v1::ai_message_service_client::AiMessageServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> ai_v1::ListAiMessageResponse,
            pass delete(ai_v1::DeleteAiMessageRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `AiProviderServiceHandlers`.
    proto::gen_admin::services::AiProviderServiceHandlers for AiProviderProxy {
        client: ai_v1::ai_provider_service_client::AiProviderServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> ai_v1::ListAiProviderResponse,
            pass get(ai_v1::GetAiProviderRequest) -> ai_v1::AiProvider,
            pass create(ai_v1::CreateAiProviderRequest) -> pbjson_types::Empty,
            pass update(ai_v1::UpdateAiProviderRequest) -> pbjson_types::Empty,
            pass delete(ai_v1::DeleteAiProviderRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `AiQueryServiceHandlers`.
    proto::gen_admin::services::AiQueryServiceHandlers for AiQueryProxy {
        client: ai_v1::ai_query_service_client::AiQueryServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass ask(ai_v1::AskAiQueryRequest) -> ai_v1::AskAiQueryResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `AiUsageLogServiceHandlers`.
    proto::gen_admin::services::AiUsageLogServiceHandlers for AiUsageLogProxy {
        client: ai_v1::ai_usage_log_service_client::AiUsageLogServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> ai_v1::ListAiUsageLogResponse,
            pass get_usage_summary(pbjson_types::Empty) -> ai_v1::UsageSummaryResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `ApiAuditLogServiceHandlers`.
    proto::gen_admin::services::ApiAuditLogServiceHandlers for ApiAuditLogProxy {
        client: audit_v1::api_audit_log_service_client::ApiAuditLogServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> audit_v1::ListApiAuditLogResponse,
            pass get(audit_v1::GetApiAuditLogRequest) -> audit_v1::ApiAuditLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `ApiServiceHandlers`.
    proto::gen_admin::services::ApiServiceHandlers for ApiProxy {
        client: permission_v1::api_service_client::ApiServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> permission_v1::ListApiResponse,
            pass get(permission_v1::GetApiRequest) -> permission_v1::Api,
            pass create(permission_v1::CreateApiRequest) -> pbjson_types::Empty,
            pass update(permission_v1::UpdateApiRequest) -> pbjson_types::Empty,
            pass delete(permission_v1::DeleteApiRequest) -> pbjson_types::Empty,
            hand sync_apis(pbjson_types::Empty) -> pbjson_types::Empty => sync_apis,
            hand get_walk_route_data(pbjson_types::Empty)
                -> permission_v1::ListApiResponse => walk_route_data,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `BackupServiceHandlers`.
    proto::gen_admin::services::BackupServiceHandlers for BackupProxy {
        client: task_v1::backup_archive_service_client::BackupArchiveServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list_backups(task_v1::ListBackupsRequest) -> task_v1::ListBackupsResponse,
            pass download_backup(task_v1::DownloadBackupRequest) -> task_v1::DownloadBackupResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `CategoryServiceHandlers`.
    proto::gen_admin::services::CategoryServiceHandlers for CategoryProxy {
        client: content_v1::category_service_client::CategoryServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> content_v1::ListCategoryResponse,
            pass get(content_v1::GetCategoryRequest) -> content_v1::Category,
            pass create(content_v1::CreateCategoryRequest) -> content_v1::Category,
            pass update(content_v1::UpdateCategoryRequest) -> content_v1::Category,
            pass delete(content_v1::DeleteCategoryRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `CommentServiceHandlers`.
    proto::gen_admin::services::CommentServiceHandlers for CommentProxy {
        client: comment_v1::comment_service_client::CommentServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> comment_v1::ListCommentResponse,
            pass get(comment_v1::GetCommentRequest) -> comment_v1::Comment,
            pass create(comment_v1::CreateCommentRequest) -> comment_v1::Comment,
            pass update(comment_v1::UpdateCommentRequest) -> comment_v1::Comment,
            pass delete(comment_v1::DeleteCommentRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `ConfigServiceHandlers`.
    proto::gen_admin::services::ConfigServiceHandlers for ConfigProxy {
        client: config_v1::config_service_client::ConfigServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> config_v1::ListConfigResponse,
            pass get(config_v1::GetConfigRequest) -> config_v1::Config,
            pass create(config_v1::CreateConfigRequest) -> pbjson_types::Empty,
            pass update(config_v1::UpdateConfigRequest) -> pbjson_types::Empty,
            pass delete(config_v1::DeleteConfigRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `ContentModelServiceHandlers`.
    proto::gen_admin::services::ContentModelServiceHandlers for ContentModelProxy {
        client: content_v1::content_model_service_client::ContentModelServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> content_v1::ListContentModelResponse,
            pass get(content_v1::GetContentModelRequest) -> content_v1::ContentModel,
            pass create(content_v1::CreateContentModelRequest) -> content_v1::ContentModel,
            pass update(content_v1::UpdateContentModelRequest) -> content_v1::ContentModel,
            pass delete(content_v1::DeleteContentModelRequest) -> pbjson_types::Empty,
            pass list_field_definitions(content_v1::ListFieldDefinitionsRequest)
                -> content_v1::ListFieldDefinitionsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `DataAccessAuditLogServiceHandlers`.
    proto::gen_admin::services::DataAccessAuditLogServiceHandlers for DataAccessAuditLogProxy {
        client: audit_v1::data_access_audit_log_service_client::DataAccessAuditLogServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> audit_v1::ListDataAccessAuditLogResponse,
            pass get(audit_v1::GetDataAccessAuditLogRequest) -> audit_v1::DataAccessAuditLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `DictEntryServiceHandlers`.
    proto::gen_admin::services::DictEntryServiceHandlers for DictEntryProxy {
        client: dict_v1::dict_entry_service_client::DictEntryServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> dict_v1::ListDictEntryResponse,
            pass create(dict_v1::CreateDictEntryRequest) -> pbjson_types::Empty,
            pass update(dict_v1::UpdateDictEntryRequest) -> pbjson_types::Empty,
            pass delete(dict_v1::DeleteDictEntryRequest) -> pbjson_types::Empty,
            pass list_by_type_code(dict_v1::ListDictEntryByTypeCodeRequest)
                -> dict_v1::ListDictEntryByTypeCodeResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `DictTypeServiceHandlers`.
    proto::gen_admin::services::DictTypeServiceHandlers for DictTypeProxy {
        client: dict_v1::dict_type_service_client::DictTypeServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> dict_v1::ListDictTypeResponse,
            pass get(dict_v1::GetDictTypeRequest) -> dict_v1::DictType,
            pass create(dict_v1::CreateDictTypeRequest) -> pbjson_types::Empty,
            pass update(dict_v1::UpdateDictTypeRequest) -> pbjson_types::Empty,
            pass delete(dict_v1::DeleteDictTypeRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `EditLockServiceHandlers`.
    proto::gen_admin::services::EditLockServiceHandlers for EditLockProxy {
        client: content_v1::edit_lock_service_client::EditLockServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass acquire(content_v1::AcquireEditLockRequest) -> content_v1::EditLock,
            pass release(content_v1::ReleaseEditLockRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `ExperimentServiceHandlers`.
    proto::gen_admin::services::ExperimentServiceHandlers for ExperimentProxy {
        client: content_v1::experiment_service_client::ExperimentServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> content_v1::ListExperimentsResponse,
            pass get(content_v1::GetExperimentRequest) -> content_v1::Experiment,
            pass create(content_v1::CreateExperimentRequest) -> content_v1::Experiment,
            pass update(content_v1::UpdateExperimentRequest) -> content_v1::Experiment,
            pass delete(content_v1::DeleteExperimentRequest) -> pbjson_types::Empty,
            pass set_status(content_v1::SetExperimentStatusRequest) -> content_v1::Experiment,
            pass get_variant_stats(content_v1::GetExperimentVariantStatsRequest)
                -> content_v1::ExperimentVariantStatsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `FileServiceHandlers`.
    proto::gen_admin::services::FileServiceHandlers for FileProxy {
        client: storage_v1::file_service_client::FileServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> storage_v1::ListFileResponse,
            pass get(storage_v1::GetFileRequest) -> storage_v1::File,
            hand create(storage_v1::CreateFileRequest) -> pbjson_types::Empty => file_create,
            pass update(storage_v1::UpdateFileRequest) -> pbjson_types::Empty,
            pass delete(storage_v1::DeleteFileRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `FlowRuleServiceHandlers`.
    proto::gen_admin::services::FlowRuleServiceHandlers for FlowRuleProxy {
        client: flow_v1::flow_rule_service_client::FlowRuleServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> flow_v1::ListFlowRuleResponse,
            pass get(flow_v1::GetFlowRuleRequest) -> flow_v1::FlowRule,
            pass create(flow_v1::CreateFlowRuleRequest) -> flow_v1::FlowRule,
            pass update(flow_v1::UpdateFlowRuleRequest) -> flow_v1::FlowRule,
            pass delete(flow_v1::DeleteFlowRuleRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `FormSubmissionServiceHandlers`.
    proto::gen_admin::services::FormSubmissionServiceHandlers for FormSubmissionProxy {
        client: content_v1::form_submission_service_client::FormSubmissionServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> content_v1::ListFormSubmissionsResponse,
            pass get(content_v1::GetFormSubmissionRequest) -> content_v1::FormSubmission,
            pass delete(content_v1::DeleteFormSubmissionRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `InteractionAdminServiceHandlers`.
    proto::gen_admin::services::InteractionAdminServiceHandlers for InteractionAdminProxy {
        client: interaction_v1::interaction_admin_service_client::InteractionAdminServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass purge_target_interactions(interaction_v1::PurgeTargetInteractionsRequest)
                -> interaction_v1::PurgeTargetInteractionsResponse,
            pass purge_user_interactions(interaction_v1::PurgeUserInteractionsRequest)
                -> interaction_v1::PurgeUserInteractionsResponse,
            pass reset_counter(interaction_v1::ResetCounterRequest)
                -> interaction_v1::ResetCounterResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `InternalMessageCategoryServiceHandlers`.
    proto::gen_admin::services::InternalMessageCategoryServiceHandlers
        for InternalMessageCategoryProxy {
        client: internal_message_v1::internal_message_category_service_client::
            InternalMessageCategoryServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest)
                -> internal_message_v1::ListInternalMessageCategoryResponse,
            pass get(internal_message_v1::GetInternalMessageCategoryRequest)
                -> internal_message_v1::InternalMessageCategory,
            pass create(internal_message_v1::CreateInternalMessageCategoryRequest)
                -> pbjson_types::Empty,
            pass update(internal_message_v1::UpdateInternalMessageCategoryRequest)
                -> pbjson_types::Empty,
            pass delete(internal_message_v1::DeleteInternalMessageCategoryRequest)
                -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `InternalMessageRecipientServiceHandlers`.
    proto::gen_admin::services::InternalMessageRecipientServiceHandlers
        for InternalMessageRecipientProxy {
        client: internal_message_v1::internal_message_recipient_service_client::
            InternalMessageRecipientServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list_user_inbox(pagination::PagingRequest)
                -> internal_message_v1::ListUserInboxResponse,
            pass delete_notification_from_inbox(
                internal_message_v1::DeleteNotificationFromInboxRequest
            ) -> pbjson_types::Empty,
            pass mark_notification_as_read(internal_message_v1::MarkNotificationAsReadRequest)
                -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `InternalMessageServiceHandlers`.
    proto::gen_admin::services::InternalMessageServiceHandlers for InternalMessageProxy {
        client: internal_message_v1::internal_message_service_client::InternalMessageServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list_message(pagination::PagingRequest)
                -> internal_message_v1::ListInternalMessageResponse,
            pass get_message(internal_message_v1::GetInternalMessageRequest)
                -> internal_message_v1::InternalMessage,
            pass update_message(internal_message_v1::UpdateInternalMessageRequest)
                -> pbjson_types::Empty,
            pass delete_message(internal_message_v1::DeleteInternalMessageRequest)
                -> pbjson_types::Empty,
            pass send_message(internal_message_v1::SendMessageRequest)
                -> internal_message_v1::SendMessageResponse,
            pass revoke_message(internal_message_v1::RevokeMessageRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `LanguageServiceHandlers`.
    proto::gen_admin::services::LanguageServiceHandlers for LanguageProxy {
        client: dict_v1::language_service_client::LanguageServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> dict_v1::ListLanguageResponse,
            pass get(dict_v1::GetLanguageRequest) -> dict_v1::Language,
            pass create(dict_v1::CreateLanguageRequest) -> pbjson_types::Empty,
            pass update(dict_v1::UpdateLanguageRequest) -> pbjson_types::Empty,
            pass delete(dict_v1::DeleteLanguageRequest) -> pbjson_types::Empty,
            pass batch_create(dict_v1::BatchCreateLanguagesRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `LoginAuditLogServiceHandlers`.
    proto::gen_admin::services::LoginAuditLogServiceHandlers for LoginAuditLogProxy {
        client: audit_v1::login_audit_log_service_client::LoginAuditLogServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> audit_v1::ListLoginAuditLogResponse,
            pass get(audit_v1::GetLoginAuditLogRequest) -> audit_v1::LoginAuditLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `LoginPolicyServiceHandlers`.
    proto::gen_admin::services::LoginPolicyServiceHandlers for LoginPolicyProxy {
        client: authentication_v1::login_policy_service_client::LoginPolicyServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> authentication_v1::ListLoginPolicyResponse,
            pass get(authentication_v1::GetLoginPolicyRequest) -> authentication_v1::LoginPolicy,
            pass create(authentication_v1::CreateLoginPolicyRequest) -> pbjson_types::Empty,
            pass update(authentication_v1::UpdateLoginPolicyRequest) -> pbjson_types::Empty,
            pass delete(authentication_v1::DeleteLoginPolicyRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `MFAServiceHandlers`.
    proto::gen_admin::services::MFAServiceHandlers for MFAProxy {
        client: authentication_v1::mfa_service_client::MfaServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            renamed get_m_f_a_status(authentication_v1::GetMfaStatusRequest)
                -> authentication_v1::GetMfaStatusResponse => get_mfa_status,
            pass start_enroll_method(authentication_v1::StartEnrollMethodRequest)
                -> authentication_v1::StartEnrollMethodResponse,
            pass confirm_enroll_method(authentication_v1::ConfirmEnrollMethodRequest)
                -> authentication_v1::ConfirmEnrollMethodResponse,
            renamed disable_m_f_a(authentication_v1::DisableMfaRequest)
                -> pbjson_types::Empty => disable_mfa,
            hand verify_m_f_a_challenge(authentication_v1::VerifyMfaChallengeRequest)
                -> authentication_v1::LoginResponse => mfa_verify_challenge,
            renamed reset_user_m_f_a(authentication_v1::ResetUserMfaRequest)
                -> pbjson_types::Empty => reset_user_mfa,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `MediaAssetServiceHandlers`.
    proto::gen_admin::services::MediaAssetServiceHandlers for MediaAssetProxy {
        client: media_v1::media_asset_service_client::MediaAssetServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> media_v1::ListMediaAssetResponse,
            pass get(media_v1::GetMediaAssetRequest) -> media_v1::MediaAsset,
            pass create(media_v1::CreateMediaAssetRequest) -> media_v1::MediaAsset,
            pass update(media_v1::UpdateMediaAssetRequest) -> media_v1::MediaAsset,
            pass find_by_hash(media_v1::FindMediaAssetByHashRequest)
                -> media_v1::FindMediaAssetByHashResponse,
            pass list_duplicated_media_assets(media_v1::ListDuplicatedMediaAssetsRequest)
                -> media_v1::ListDuplicatedMediaAssetsResponse,
            pass list_media_asset_references(media_v1::ListMediaAssetReferencesRequest)
                -> media_v1::ListMediaAssetReferencesResponse,
            pass delete(media_v1::DeleteMediaAssetRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `MenuServiceHandlers`.
    proto::gen_admin::services::MenuServiceHandlers for MenuProxy {
        client: permission_v1::menu_service_client::MenuServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> permission_v1::ListMenuResponse,
            pass get(permission_v1::GetMenuRequest) -> permission_v1::Menu,
            pass create(permission_v1::CreateMenuRequest) -> pbjson_types::Empty,
            pass update(permission_v1::UpdateMenuRequest) -> pbjson_types::Empty,
            pass delete(permission_v1::DeleteMenuRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `MonitorAlertServiceHandlers`.
    proto::gen_admin::services::MonitorAlertServiceHandlers for MonitorAlertProxy {
        client: monitor_alert_v1::monitor_alert_service_client::MonitorAlertServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list_monitor_alert_rule(pagination::PagingRequest)
                -> monitor_alert_v1::ListMonitorAlertRuleResponse,
            pass get_monitor_alert_rule(monitor_alert_v1::GetMonitorAlertRuleRequest)
                -> monitor_alert_v1::MonitorAlertRule,
            pass create_monitor_alert_rule(monitor_alert_v1::CreateMonitorAlertRuleRequest)
                -> monitor_alert_v1::MonitorAlertRule,
            pass update_monitor_alert_rule(monitor_alert_v1::UpdateMonitorAlertRuleRequest)
                -> pbjson_types::Empty,
            pass delete_monitor_alert_rule(monitor_alert_v1::DeleteMonitorAlertRuleRequest)
                -> pbjson_types::Empty,
            pass evaluate_monitor_alerts(monitor_alert_v1::EvaluateMonitorAlertsRequest)
                -> monitor_alert_v1::EvaluateMonitorAlertsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `NavigationItemServiceHandlers`.
    proto::gen_admin::services::NavigationItemServiceHandlers for NavigationItemProxy {
        client: site_v1::navigation_item_service_client::NavigationItemServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> site_v1::ListNavigationItemResponse,
            pass get(site_v1::GetNavigationItemRequest) -> site_v1::NavigationItem,
            pass create(site_v1::CreateNavigationItemRequest) -> site_v1::NavigationItem,
            pass update(site_v1::UpdateNavigationItemRequest) -> site_v1::NavigationItem,
            pass delete(site_v1::DeleteNavigationItemRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `NavigationServiceHandlers`.
    proto::gen_admin::services::NavigationServiceHandlers for NavigationProxy {
        client: site_v1::navigation_service_client::NavigationServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> site_v1::ListNavigationResponse,
            pass get(site_v1::GetNavigationRequest) -> site_v1::Navigation,
            pass create(site_v1::CreateNavigationRequest) -> site_v1::Navigation,
            pass update(site_v1::UpdateNavigationRequest) -> site_v1::Navigation,
            pass delete(site_v1::DeleteNavigationRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `NotificationChannelServiceHandlers`.
    proto::gen_admin::services::NotificationChannelServiceHandlers for NotificationChannelProxy {
        client: notify_v1::notification_channel_service_client::NotificationChannelServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> notify_v1::ListNotificationChannelResponse,
            pass get(notify_v1::GetNotificationChannelRequest) -> notify_v1::NotificationChannel,
            pass create(notify_v1::CreateNotificationChannelRequest)
                -> notify_v1::NotificationChannel,
            pass update(notify_v1::UpdateNotificationChannelRequest)
                -> notify_v1::NotificationChannel,
            pass delete(notify_v1::DeleteNotificationChannelRequest) -> pbjson_types::Empty,
            pass send_test_email(notify_v1::SendTestEmailRequest)
                -> notify_v1::SendTestEmailResponse,
            pass list_channel_schemas(pbjson_types::Empty) -> notify_v1::ListChannelSchemasResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `NotificationPreferenceServiceHandlers`.
    proto::gen_admin::services::NotificationPreferenceServiceHandlers
        for NotificationPreferenceProxy {
        client: notify_v1::notification_preference_service_client::
            NotificationPreferenceServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass get_my_notification_preference(pbjson_types::Empty)
                -> notify_v1::NotificationPreference,
            pass update_my_notification_preference(notify_v1::UpdateNotificationPreferenceRequest)
                -> notify_v1::NotificationPreference,
            pass list_my_notification_categories(pbjson_types::Empty)
                -> notify_v1::ListMyNotificationCategoriesResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `NotificationRuleServiceHandlers`.
    proto::gen_admin::services::NotificationRuleServiceHandlers for NotificationRuleProxy {
        client: notify_v1::notification_rule_service_client::NotificationRuleServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> notify_v1::ListNotificationRuleResponse,
            pass get(notify_v1::GetNotificationRuleRequest) -> notify_v1::NotificationRule,
            pass create(notify_v1::CreateNotificationRuleRequest) -> notify_v1::NotificationRule,
            pass update(notify_v1::UpdateNotificationRuleRequest) -> notify_v1::NotificationRule,
            pass delete(notify_v1::DeleteNotificationRuleRequest) -> pbjson_types::Empty,
            pass test_dispatch(notify_v1::TestDispatchNotificationRequest)
                -> notify_v1::TestDispatchNotificationResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `NotificationServiceHandlers`.
    proto::gen_admin::services::NotificationServiceHandlers for NotificationProxy {
        client: notify_v1::notification_service_client::NotificationServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list_delivery(pagination::PagingRequest)
                -> notify_v1::ListNotificationDeliveryResponse,
            pass get_delivery(notify_v1::GetNotificationDeliveryRequest)
                -> notify_v1::NotificationDelivery,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `NotificationTemplateServiceHandlers`.
    proto::gen_admin::services::NotificationTemplateServiceHandlers for NotificationTemplateProxy {
        client: notify_v1::notification_template_service_client::NotificationTemplateServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> notify_v1::ListNotificationTemplateResponse,
            pass get(notify_v1::GetNotificationTemplateRequest) -> notify_v1::NotificationTemplate,
            pass create(notify_v1::CreateNotificationTemplateRequest)
                -> notify_v1::NotificationTemplate,
            pass update(notify_v1::UpdateNotificationTemplateRequest)
                -> notify_v1::NotificationTemplate,
            pass delete(notify_v1::DeleteNotificationTemplateRequest) -> pbjson_types::Empty,
            pass render(notify_v1::RenderNotificationTemplateRequest)
                -> notify_v1::RenderNotificationTemplateResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `OnlineSessionServiceHandlers`.
    proto::gen_admin::services::OnlineSessionServiceHandlers for OnlineSessionProxy {
        client: online_session_v1::online_session_service_client::OnlineSessionServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            renamed list_online_session(online_session_v1::ListOnlineSessionRequest)
                -> online_session_v1::ListOnlineSessionResponse => list,
            renamed force_logout_session(online_session_v1::ForceLogoutSessionRequest)
                -> online_session_v1::ForceLogoutSessionResponse => force_logout,
            renamed list_my_online_session(online_session_v1::ListMyOnlineSessionRequest)
                -> online_session_v1::ListOnlineSessionResponse => list_my,
            renamed revoke_my_online_session(online_session_v1::RevokeMyOnlineSessionRequest)
                -> online_session_v1::RevokeMyOnlineSessionResponse => revoke_my,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `OperationAuditLogServiceHandlers`.
    proto::gen_admin::services::OperationAuditLogServiceHandlers for OperationAuditLogProxy {
        client: audit_v1::operation_audit_log_service_client::OperationAuditLogServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> audit_v1::ListOperationAuditLogResponse,
            pass get(audit_v1::GetOperationAuditLogRequest) -> audit_v1::OperationAuditLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `OrgUnitServiceHandlers`.
    proto::gen_admin::services::OrgUnitServiceHandlers for OrgUnitProxy {
        client: identity_v1::org_unit_service_client::OrgUnitServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> identity_v1::ListOrgUnitResponse,
            pass get(identity_v1::GetOrgUnitRequest) -> identity_v1::OrgUnit,
            pass create(identity_v1::CreateOrgUnitRequest) -> pbjson_types::Empty,
            pass update(identity_v1::UpdateOrgUnitRequest) -> pbjson_types::Empty,
            pass delete(identity_v1::DeleteOrgUnitRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PageServiceHandlers`.
    proto::gen_admin::services::PageServiceHandlers for PageProxy {
        client: content_v1::page_service_client::PageServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> content_v1::ListPageResponse,
            pass get(content_v1::GetPageRequest) -> content_v1::Page,
            pass create(content_v1::CreatePageRequest) -> content_v1::Page,
            pass update(content_v1::UpdatePageRequest) -> content_v1::Page,
            pass delete(content_v1::DeletePageRequest) -> pbjson_types::Empty,
            pass list_versions(content_v1::ListPageVersionsRequest)
                -> content_v1::ListPageVersionsResponse,
            pass get_version(content_v1::GetPageVersionRequest) -> content_v1::PageVersion,
            pass restore_version(content_v1::RestorePageVersionRequest) -> content_v1::Page,
            pass submit_review(content_v1::SubmitPageReviewRequest) -> content_v1::Page,
            pass approve_review(content_v1::ReviewPageRequest) -> content_v1::Page,
            pass reject_review(content_v1::RejectPageReviewRequest) -> content_v1::Page,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PermissionAuditLogServiceHandlers`.
    proto::gen_admin::services::PermissionAuditLogServiceHandlers for PermissionAuditLogProxy {
        client: audit_v1::permission_audit_log_service_client::PermissionAuditLogServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> audit_v1::ListPermissionAuditLogResponse,
            pass get(audit_v1::GetPermissionAuditLogRequest) -> audit_v1::PermissionAuditLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PermissionGroupServiceHandlers`.
    proto::gen_admin::services::PermissionGroupServiceHandlers for PermissionGroupProxy {
        client: permission_v1::permission_group_service_client::PermissionGroupServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> permission_v1::ListPermissionGroupResponse,
            pass get(permission_v1::GetPermissionGroupRequest) -> permission_v1::PermissionGroup,
            pass create(permission_v1::CreatePermissionGroupRequest) -> pbjson_types::Empty,
            pass update(permission_v1::UpdatePermissionGroupRequest) -> pbjson_types::Empty,
            pass delete(permission_v1::DeletePermissionGroupRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PermissionServiceHandlers`.
    proto::gen_admin::services::PermissionServiceHandlers for PermissionProxy {
        client: permission_v1::permission_service_client::PermissionServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> permission_v1::ListPermissionResponse,
            pass get(permission_v1::GetPermissionRequest) -> permission_v1::Permission,
            pass create(permission_v1::CreatePermissionRequest) -> pbjson_types::Empty,
            pass update(permission_v1::UpdatePermissionRequest) -> pbjson_types::Empty,
            pass delete(permission_v1::DeletePermissionRequest) -> pbjson_types::Empty,
            hand sync_permissions(pbjson_types::Empty) -> pbjson_types::Empty => sync_permissions,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PolicyEvaluationLogServiceHandlers`.
    proto::gen_admin::services::PolicyEvaluationLogServiceHandlers for PolicyEvaluationLogProxy {
        client: permission_v1::policy_evaluation_log_service_client::
            PolicyEvaluationLogServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> permission_v1::ListPolicyEvaluationLogResponse,
            pass get(permission_v1::GetPolicyEvaluationLogRequest)
                -> permission_v1::PolicyEvaluationLog,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PositionServiceHandlers`.
    proto::gen_admin::services::PositionServiceHandlers for PositionProxy {
        client: identity_v1::position_service_client::PositionServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> identity_v1::ListPositionResponse,
            pass get(identity_v1::GetPositionRequest) -> identity_v1::Position,
            pass create(identity_v1::CreatePositionRequest) -> pbjson_types::Empty,
            pass update(identity_v1::UpdatePositionRequest) -> pbjson_types::Empty,
            pass delete(identity_v1::DeletePositionRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `PostServiceHandlers`.
    proto::gen_admin::services::PostServiceHandlers for PostProxy {
        client: content_v1::post_service_client::PostServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> content_v1::ListPostResponse,
            pass get(content_v1::GetPostRequest) -> content_v1::Post,
            pass create(content_v1::CreatePostRequest) -> content_v1::Post,
            pass update(content_v1::UpdatePostRequest) -> content_v1::Post,
            pass delete(content_v1::DeletePostRequest) -> pbjson_types::Empty,
            pass translation_exists(content_v1::PostTranslationExistsRequest)
                -> content_v1::PostTranslationExistsResponse,
            pass list_versions(content_v1::ListPostVersionsRequest)
                -> content_v1::ListPostVersionsResponse,
            pass get_version(content_v1::GetPostVersionRequest) -> content_v1::PostVersion,
            pass restore_version(content_v1::RestorePostVersionRequest) -> content_v1::Post,
            pass submit_review(content_v1::SubmitPostReviewRequest) -> content_v1::Post,
            pass approve_review(content_v1::ReviewPostRequest) -> content_v1::Post,
            pass reject_review(content_v1::RejectPostReviewRequest) -> content_v1::Post,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `RedirectServiceHandlers`.
    proto::gen_admin::services::RedirectServiceHandlers for RedirectProxy {
        client: content_v1::redirect_service_client::RedirectServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> content_v1::ListRedirectsResponse,
            pass get(content_v1::GetRedirectRequest) -> content_v1::Redirect,
            pass create(content_v1::CreateRedirectRequest) -> content_v1::Redirect,
            pass update(content_v1::UpdateRedirectRequest) -> content_v1::Redirect,
            pass delete(content_v1::DeleteRedirectRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `RedisCacheMonitorServiceHandlers`.
    proto::gen_admin::services::RedisCacheMonitorServiceHandlers for RedisCacheMonitorProxy {
        client: redis_cache_v1::redis_cache_monitor_service_client::RedisCacheMonitorServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass get(redis_cache_v1::GetRedisCacheMonitorRequest)
                -> redis_cache_v1::RedisCacheMonitorInfo,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `ReviewLogServiceHandlers`.
    proto::gen_admin::services::ReviewLogServiceHandlers for ReviewLogProxy {
        client: content_v1::review_log_service_client::ReviewLogServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(content_v1::ListReviewLogsRequest) -> content_v1::ListReviewLogsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `RoleServiceHandlers`.
    proto::gen_admin::services::RoleServiceHandlers for RoleProxy {
        client: permission_v1::role_service_client::RoleServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> permission_v1::ListRoleResponse,
            pass get(permission_v1::GetRoleRequest) -> permission_v1::Role,
            pass create(permission_v1::CreateRoleRequest) -> pbjson_types::Empty,
            pass update(permission_v1::UpdateRoleRequest) -> pbjson_types::Empty,
            pass delete(permission_v1::DeleteRoleRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `SearchSynonymServiceHandlers`.
    proto::gen_admin::services::SearchSynonymServiceHandlers for SearchSynonymProxy {
        client: content_v1::search_synonym_service_client::SearchSynonymServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pbjson_types::Empty) -> content_v1::ListSearchSynonymsResponse,
            pass create(content_v1::CreateSearchSynonymRequest) -> content_v1::SearchSynonym,
            pass delete(content_v1::DeleteSearchSynonymRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `ServerMonitorServiceHandlers`.
    proto::gen_admin::services::ServerMonitorServiceHandlers for ServerMonitorProxy {
        client: server_monitor_v1::server_monitor_service_client::ServerMonitorServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass get(server_monitor_v1::GetServerMonitorRequest)
                -> server_monitor_v1::ServerMonitorInfo,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `SiteServiceHandlers`.
    proto::gen_admin::services::SiteServiceHandlers for SiteProxy {
        client: site_v1::site_service_client::SiteServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> site_v1::ListSiteResponse,
            pass get(site_v1::GetSiteRequest) -> site_v1::Site,
            pass create(site_v1::CreateSiteRequest) -> site_v1::Site,
            drop update(site_v1::UpdateSiteRequest) -> site_v1::Site,
            pass delete(site_v1::DeleteSiteRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `SiteSettingServiceHandlers`.
    proto::gen_admin::services::SiteSettingServiceHandlers for SiteSettingProxy {
        client: site_v1::site_setting_service_client::SiteSettingServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> site_v1::ListSiteSettingResponse,
            pass get(site_v1::GetSiteSettingRequest) -> site_v1::SiteSetting,
            pass create(site_v1::CreateSiteSettingRequest) -> site_v1::SiteSetting,
            pass update(site_v1::UpdateSiteSettingRequest) -> site_v1::SiteSetting,
            pass delete(site_v1::DeleteSiteSettingRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `SocialServiceHandlers`.
    proto::gen_admin::services::SocialServiceHandlers for SocialProxy {
        client: social_v1::social_service_client::SocialServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list_social_accounts(pagination::PagingRequest)
                -> social_v1::ListSocialAccountsResponse,
            pass get_social_account(social_v1::GetSocialAccountRequest) -> social_v1::SocialAccount,
            pass create_social_account(social_v1::CreateSocialAccountRequest)
                -> social_v1::SocialAccount,
            pass update_social_account(social_v1::UpdateSocialAccountRequest)
                -> social_v1::SocialAccount,
            pass delete_social_account(social_v1::DeleteSocialAccountRequest)
                -> pbjson_types::Empty,
            pass list_social_distributions(pagination::PagingRequest)
                -> social_v1::ListSocialDistributionsResponse,
            pass get_social_distribution(social_v1::GetSocialDistributionRequest)
                -> social_v1::SocialDistribution,
            pass create_social_distribution(social_v1::CreateSocialDistributionRequest)
                -> social_v1::SocialDistribution,
            pass update_social_distribution(social_v1::UpdateSocialDistributionRequest)
                -> social_v1::SocialDistribution,
            pass delete_social_distribution(social_v1::DeleteSocialDistributionRequest)
                -> pbjson_types::Empty,
            pass publish_social_distribution(social_v1::PublishSocialDistributionRequest)
                -> social_v1::SocialDistribution,
            pass refresh_social_distribution_stats(social_v1::RefreshSocialDistributionStatsRequest)
                -> social_v1::SocialDistribution,
            pass list_social_distribution_stats_history(
                social_v1::ListSocialDistributionStatsHistoryRequest
            ) -> social_v1::ListSocialDistributionStatsHistoryResponse,
            pass get_social_distribution_summary(pbjson_types::Empty)
                -> social_v1::SocialDistributionSummary,
            pass move_social_distribution(social_v1::MoveSocialDistributionRequest)
                -> social_v1::SocialDistribution,
            pass list_social_inbox(social_v1::ListSocialInboxRequest)
                -> social_v1::ListSocialInboxResponse,
            pass mark_social_inbox_read(social_v1::MarkSocialInboxReadRequest)
                -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `StatsServiceHandlers`.
    proto::gen_admin::services::StatsServiceHandlers for StatsProxy {
        client: stats_v1::stats_service_client::StatsServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass get_dashboard_overview(stats_v1::GetDashboardOverviewRequest)
                -> stats_v1::GetDashboardOverviewResponse,
            pass get_content_trend(stats_v1::GetContentTrendRequest)
                -> stats_v1::GetContentTrendResponse,
            pass get_interaction_stats(stats_v1::GetInteractionStatsRequest)
                -> stats_v1::GetInteractionStatsResponse,
            pass get_login_activity(stats_v1::GetLoginActivityRequest)
                -> stats_v1::GetLoginActivityResponse,
            pass get_ai_usage_trend(stats_v1::TrendRequest) -> stats_v1::GetAiUsageTrendResponse,
            pass get_api_call_trend(stats_v1::TrendRequest) -> stats_v1::GetApiCallTrendResponse,
            pass get_post_status_distribution(stats_v1::GetPostStatusDistributionRequest)
                -> stats_v1::GetPostStatusDistributionResponse,
            pass get_comment_status_distribution(stats_v1::GetCommentStatusDistributionRequest)
                -> stats_v1::GetCommentStatusDistributionResponse,
            pass get_media_asset_type_distribution(stats_v1::GetMediaAssetTypeDistributionRequest)
                -> stats_v1::GetMediaAssetTypeDistributionResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `TagServiceHandlers`.
    proto::gen_admin::services::TagServiceHandlers for TagProxy {
        client: content_v1::tag_service_client::TagServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> content_v1::ListTagResponse,
            pass get(content_v1::GetTagRequest) -> content_v1::Tag,
            pass create(content_v1::CreateTagRequest) -> content_v1::Tag,
            pass update(content_v1::UpdateTagRequest) -> content_v1::Tag,
            pass delete(content_v1::DeleteTagRequest) -> pbjson_types::Empty,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `TaskServiceHandlers`.
    proto::gen_admin::services::TaskServiceHandlers for TaskProxy {
        client: task_v1::task_service_client::TaskServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> task_v1::ListTaskResponse,
            pass get(task_v1::GetTaskRequest) -> task_v1::Task,
            pass create(task_v1::CreateTaskRequest) -> pbjson_types::Empty,
            pass update(task_v1::UpdateTaskRequest) -> pbjson_types::Empty,
            pass delete(task_v1::DeleteTaskRequest) -> pbjson_types::Empty,
            pass list_task_type_name(pbjson_types::Empty) -> task_v1::ListTaskTypeNameResponse,
            pass restart_all_task(pbjson_types::Empty) -> task_v1::RestartAllTaskResponse,
            pass start_all_task(pbjson_types::Empty) -> pbjson_types::Empty,
            pass stop_all_task(pbjson_types::Empty) -> pbjson_types::Empty,
            pass control_task(task_v1::ControlTaskRequest) -> pbjson_types::Empty,
            pass list_task_executions(task_v1::ListTaskExecutionsRequest)
                -> task_v1::ListTaskExecutionsResponse,
            pass inspect_system_tasks(task_v1::InspectSystemTasksRequest)
                -> task_v1::InspectSystemTasksResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `TenantQuotaServiceHandlers`.
    proto::gen_admin::services::TenantQuotaServiceHandlers for TenantQuotaProxy {
        client: identity_v1::tenant_quota_service_client::TenantQuotaServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> identity_v1::ListTenantQuotaResponse,
            pass get(identity_v1::GetTenantQuotaRequest) -> identity_v1::TenantQuota,
            pass create(identity_v1::CreateTenantQuotaRequest) -> pbjson_types::Empty,
            pass update(identity_v1::UpdateTenantQuotaRequest) -> pbjson_types::Empty,
            pass delete(identity_v1::DeleteTenantQuotaRequest) -> pbjson_types::Empty,
            pass get_usage(identity_v1::GetTenantUsageRequest) -> identity_v1::TenantUsage,
            pass get_my_tenant_usage(identity_v1::GetMyTenantUsageRequest)
                -> identity_v1::TenantUsage,
            pass list_quota_plans(identity_v1::ListQuotaPlansRequest)
                -> identity_v1::ListQuotaPlansResponse,
            pass create_quota_plan(identity_v1::CreateQuotaPlanRequest) -> identity_v1::QuotaPlan,
            pass delete_quota_plan(identity_v1::DeleteQuotaPlanRequest) -> pbjson_types::Empty,
            pass apply_quota_plan(identity_v1::ApplyQuotaPlanRequest)
                -> identity_v1::ApplyQuotaPlanResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `TenantServiceHandlers`.
    proto::gen_admin::services::TenantServiceHandlers for TenantProxy {
        client: identity_v1::tenant_service_client::TenantServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> identity_v1::ListTenantResponse,
            pass get(identity_v1::GetTenantRequest) -> identity_v1::Tenant,
            drop create(identity_v1::CreateTenantRequest) -> pbjson_types::Empty,
            pass update(identity_v1::UpdateTenantRequest) -> pbjson_types::Empty,
            pass delete(identity_v1::DeleteTenantRequest) -> pbjson_types::Empty,
            pass create_tenant_with_admin_user(identity_v1::CreateTenantWithAdminUserRequest)
                -> pbjson_types::Empty,
            pass tenant_exists(identity_v1::TenantExistsRequest)
                -> identity_v1::TenantExistsResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `TranslatorServiceHandlers`.
    proto::gen_admin::services::TranslatorServiceHandlers for TranslatorProxy {
        client: translator_v1::translator_service_client::TranslatorServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass translate(translator_v1::TranslateRequest) -> translator_v1::TranslateResponse,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `UserProfileServiceHandlers`.
    proto::gen_admin::services::UserProfileServiceHandlers for UserProfileProxy {
        client: identity_v1::user_profile_service_client::UserProfileServiceClient<
            tonic::transport::Channel,
        >,
        behaviors: behaviors,
        methods: [
            pass get_user(pbjson_types::Empty) -> identity_v1::User,
            pass update_user(identity_v1::UpdateUserRequest) -> pbjson_types::Empty,
            hand change_password(identity_v1::ChangePasswordRequest)
                -> pbjson_types::Empty => change_password,
            hand bind_contact(identity_v1::BindContactRequest)
                -> pbjson_types::Empty => bind_contact,
            hand verify_contact(identity_v1::VerifyContactRequest)
                -> pbjson_types::Empty => verify_contact,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `UserServiceHandlers`.
    proto::gen_admin::services::UserServiceHandlers for UserProxy {
        client: identity_v1::user_service_client::UserServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> identity_v1::ListUserResponse,
            pass get(identity_v1::GetUserRequest) -> identity_v1::User,
            pass create(identity_v1::CreateUserRequest) -> identity_v1::User,
            pass update(identity_v1::UpdateUserRequest) -> pbjson_types::Empty,
            pass delete(identity_v1::DeleteUserRequest) -> pbjson_types::Empty,
            pass user_exists(identity_v1::UserExistsRequest) -> identity_v1::UserExistsResponse,
            hand edit_user_password(identity_v1::EditUserPasswordRequest)
                -> pbjson_types::Empty => edit_user_password,
        ]
    }
}

passthrough_proxy! {
    /// The pass-through proxy of `WebhookServiceHandlers`.
    proto::gen_admin::services::WebhookServiceHandlers for WebhookProxy {
        client: webhook_v1::webhook_service_client::WebhookServiceClient<tonic::transport::Channel>,
        behaviors: behaviors,
        methods: [
            pass list(pagination::PagingRequest) -> webhook_v1::ListWebhookResponse,
            pass get(webhook_v1::GetWebhookRequest) -> webhook_v1::Webhook,
            pass create(webhook_v1::CreateWebhookRequest) -> webhook_v1::Webhook,
            pass update(webhook_v1::UpdateWebhookRequest) -> webhook_v1::Webhook,
            pass delete(webhook_v1::DeleteWebhookRequest) -> pbjson_types::Empty,
            pass test_ping(webhook_v1::TestWebhookPingRequest) -> webhook_v1::WebhookDelivery,
            pass list_deliveries(webhook_v1::ListWebhookDeliveriesRequest)
                -> webhook_v1::ListWebhookDeliveriesResponse,
        ]
    }
}
