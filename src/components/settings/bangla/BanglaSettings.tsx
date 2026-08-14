import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { commands } from "@/bindings";
import { Input } from "../../ui/Input";
import { Dropdown, SettingContainer, SettingsGroup } from "../../ui";
import { ApiKeyField } from "../PostProcessingSettingsApi/ApiKeyField";
import { useSettings } from "../../../hooks/useSettings";

/**
 * Checkpoint 2 configuration is deliberately independent from both selected
 * local models and English post-processing. The backend provider boundary
 * documents the small set of changes needed when another STT provider is
 * added; this UI then exposes its endpoint, model, and user-owned key.
 */
export const BanglaSettings: React.FC = () => {
  const { t } = useTranslation();
  const { settings, refreshSettings } = useSettings();
  const providerId = settings?.bangla_stt_provider_id ?? "deepgram";
  const apiKey = settings?.bangla_stt_api_keys?.[providerId] ?? "";
  const model = settings?.bangla_stt_models?.[providerId] ?? "nova-3";
  const endpoint = settings?.bangla_stt_endpoint ?? "";
  const romanizationProviderId =
    settings?.bangla_romanization_provider_id ?? "gemini";
  const romanizationApiKey =
    settings?.bangla_romanization_api_keys?.[romanizationProviderId] ?? "";
  const romanizationModel =
    settings?.bangla_romanization_models?.[romanizationProviderId] ?? "";
  const romanizationTimeout =
    settings?.bangla_romanization_timeout_seconds ?? 45;
  const [endpointDraft, setEndpointDraft] = useState(endpoint);
  const [modelDraft, setModelDraft] = useState(model);
  const [romanizationModelDraft, setRomanizationModelDraft] =
    useState(romanizationModel);
  const [romanizationTimeoutDraft, setRomanizationTimeoutDraft] = useState(
    String(romanizationTimeout),
  );

  useEffect(() => setEndpointDraft(endpoint), [endpoint]);
  useEffect(() => setModelDraft(model), [model]);
  useEffect(
    () => setRomanizationModelDraft(romanizationModel),
    [romanizationModel],
  );
  useEffect(
    () => setRomanizationTimeoutDraft(String(romanizationTimeout)),
    [romanizationTimeout],
  );

  const runUpdate = async (
    operation: Promise<{ status: string; error?: string }>,
  ) => {
    const result = await operation;
    if (result.status === "error") {
      throw new Error(result.error ?? t("bangla.errors.save"));
    }
    await refreshSettings();
  };

  const save = (operation: Promise<{ status: string; error?: string }>) => {
    void runUpdate(operation).catch((error) => {
      toast.error(t("bangla.errors.save"), {
        description: String(error),
      });
      void refreshSettings();
    });
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title={t("bangla.title")}>
        <SettingContainer
          title={t("bangla.provider.title")}
          description={t("bangla.provider.description")}
          descriptionMode="tooltip"
          layout="horizontal"
          grouped={true}
        >
          <span className="text-sm font-medium">
            {t("bangla.provider.name")}
          </span>
        </SettingContainer>

        <SettingContainer
          title={t("bangla.endpoint.title")}
          description={t("bangla.endpoint.description")}
          descriptionMode="tooltip"
          layout="stacked"
          grouped={true}
        >
          <Input
            value={endpointDraft}
            onChange={(event) => setEndpointDraft(event.target.value)}
            onBlur={() =>
              save(commands.changeBanglaSttEndpointSetting(endpointDraft))
            }
            className="w-full"
            aria-label={t("bangla.endpoint.title")}
          />
        </SettingContainer>

        <SettingContainer
          title={t("bangla.model.title")}
          description={t("bangla.model.description")}
          descriptionMode="tooltip"
          layout="horizontal"
          grouped={true}
        >
          <Input
            value={modelDraft}
            onChange={(event) => setModelDraft(event.target.value)}
            onBlur={() =>
              save(commands.changeBanglaSttModelSetting(modelDraft))
            }
            className="min-w-[260px]"
            aria-label={t("bangla.model.title")}
          />
        </SettingContainer>

        <SettingContainer
          title={t("bangla.apiKey.title")}
          description={t("bangla.apiKey.description")}
          descriptionMode="tooltip"
          layout="horizontal"
          grouped={true}
        >
          <ApiKeyField
            value={apiKey}
            onBlur={(value) =>
              save(commands.changeBanglaSttApiKeySetting(value))
            }
            placeholder={t("bangla.apiKey.placeholder")}
            disabled={false}
            className="min-w-[320px]"
          />
        </SettingContainer>
      </SettingsGroup>

      <SettingsGroup title={t("bangla.romanization.title")}>
        <SettingContainer
          title={t("bangla.romanization.provider.title")}
          description={t("bangla.romanization.provider.description")}
          descriptionMode="tooltip"
          layout="horizontal"
          grouped={true}
        >
          <Dropdown
            selectedValue={romanizationProviderId}
            options={[
              {
                value: "groq",
                label: t("bangla.romanization.provider.groq"),
              },
              {
                value: "gemini",
                label: t("bangla.romanization.provider.gemini"),
              },
              {
                value: "openai",
                label: t("bangla.romanization.provider.openai"),
              },
            ]}
            onSelect={(providerId) =>
              save(commands.changeBanglaRomanizationProviderSetting(providerId))
            }
            placeholder={t("bangla.romanization.provider.placeholder")}
          />
        </SettingContainer>

        <SettingContainer
          title={t("bangla.romanization.apiKey.title")}
          description={t("bangla.romanization.apiKey.description")}
          descriptionMode="tooltip"
          layout="horizontal"
          grouped={true}
        >
          <ApiKeyField
            value={romanizationApiKey}
            onBlur={(value) =>
              save(
                commands.changeBanglaRomanizationApiKeySetting(
                  romanizationProviderId,
                  value,
                ),
              )
            }
            placeholder={t("bangla.romanization.apiKey.placeholder")}
            disabled={false}
            className="min-w-[320px]"
          />
        </SettingContainer>

        <SettingContainer
          title={t("bangla.romanization.model.title")}
          description={t("bangla.romanization.model.description")}
          descriptionMode="tooltip"
          layout="horizontal"
          grouped={true}
        >
          <Input
            value={romanizationModelDraft}
            onChange={(event) => setRomanizationModelDraft(event.target.value)}
            onBlur={() =>
              save(
                commands.changeBanglaRomanizationModelSetting(
                  romanizationProviderId,
                  romanizationModelDraft,
                ),
              )
            }
            className="min-w-[260px]"
            aria-label={t("bangla.romanization.model.title")}
          />
        </SettingContainer>

        <SettingContainer
          title={t("bangla.romanization.timeout.title")}
          description={t("bangla.romanization.timeout.description")}
          descriptionMode="tooltip"
          layout="horizontal"
          grouped={true}
        >
          <Input
            type="number"
            min={5}
            max={120}
            value={romanizationTimeoutDraft}
            onChange={(event) =>
              setRomanizationTimeoutDraft(event.target.value)
            }
            onBlur={() => {
              const timeout = Number(romanizationTimeoutDraft);
              if (Number.isInteger(timeout)) {
                save(commands.changeBanglaRomanizationTimeoutSetting(timeout));
              } else {
                void refreshSettings();
              }
            }}
            className="w-28"
            aria-label={t("bangla.romanization.timeout.title")}
          />
        </SettingContainer>
      </SettingsGroup>

      <SettingsGroup title={t("bangla.privacy.title")}>
        <p className="text-sm text-mid-gray">
          {t("bangla.privacy.description")}
        </p>
      </SettingsGroup>
    </div>
  );
};
