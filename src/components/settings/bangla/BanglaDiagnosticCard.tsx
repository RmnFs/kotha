import React, { useEffect, useMemo, useState } from "react";
import { ChevronDown, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  commands,
  events,
  type BanglaDiagnostic,
  type BanglaDiagnosticOutcomeCategory,
} from "@/bindings";
import { Button } from "../../ui/Button";
import { SettingsGroup } from "../../ui";

const outcomeTranslationKeys: Record<BanglaDiagnosticOutcomeCategory, string> =
  {
    romanized: "bangla.diagnostics.outcomes.romanized",
    raw_bangla: "bangla.diagnostics.outcomes.rawBangla",
    romanization_fallback: "bangla.diagnostics.outcomes.fallback",
    cancelled: "bangla.diagnostics.outcomes.cancelled",
    failed: "bangla.diagnostics.outcomes.failed",
  };

const outcomeClasses: Record<BanglaDiagnosticOutcomeCategory, string> = {
  romanized: "bg-green-500/10 text-green-500",
  raw_bangla: "bg-blue-500/10 text-blue-500",
  romanization_fallback: "bg-yellow-500/10 text-yellow-500",
  cancelled: "bg-mid-gray/10 text-mid-gray",
  failed: "bg-red-500/10 text-red-400",
};

interface SummaryRowProps {
  label: string;
  duration?: string;
  metadata?: string;
}

const SummaryRow: React.FC<SummaryRowProps> = ({
  label,
  duration,
  metadata,
}) => (
  <div className="grid grid-cols-[minmax(0,1fr)_auto] gap-x-4 gap-y-0.5 py-2">
    <span className="text-sm text-text/70">{label}</span>
    {duration && (
      <span className="text-sm font-mono tabular-nums text-text">
        {duration}
      </span>
    )}
    {metadata && (
      <span className="col-span-2 text-xs text-mid-gray break-all">
        {metadata}
      </span>
    )}
  </div>
);

interface DetailRowProps {
  label: string;
  value: string;
}

const DetailRow: React.FC<DetailRowProps> = ({ label, value }) => (
  <div className="flex items-start justify-between gap-4 py-1.5">
    <span className="text-xs text-mid-gray">{label}</span>
    <span className="text-xs font-mono tabular-nums text-right break-all">
      {value}
    </span>
  </div>
);

export const BanglaDiagnosticCard: React.FC = () => {
  const { t } = useTranslation();
  const [diagnostic, setDiagnostic] = useState<BanglaDiagnostic | null>(null);
  const [showDetails, setShowDetails] = useState(false);
  const [clearing, setClearing] = useState(false);

  useEffect(() => {
    let disposed = false;
    const unlisten = events.banglaDiagnosticEvent.listen((event) => {
      if (!disposed) {
        setDiagnostic(event.payload.diagnostic);
      }
    });

    void commands.getLatestBanglaDiagnostic().then((result) => {
      if (!disposed && result.status === "ok") {
        setDiagnostic(result.data);
      }
    });

    return () => {
      disposed = true;
      void unlisten.then((stopListening) => stopListening());
    };
  }, []);

  const formatDuration = (milliseconds: number) => {
    if (milliseconds < 1_000) {
      return t("bangla.diagnostics.duration.milliseconds", {
        value: Math.round(milliseconds),
      });
    }
    return t("bangla.diagnostics.duration.seconds", {
      value: (milliseconds / 1_000).toFixed(2),
    });
  };

  const providerLabel = (provider: string) =>
    t(`bangla.diagnostics.providers.${provider}`, {
      defaultValue: provider,
    });

  const transportLabel = (transport: string) =>
    t(`bangla.diagnostics.transports.${transport}`, {
      defaultValue: transport,
    });

  const details = useMemo(() => {
    if (!diagnostic) return [];
    const optionalDuration = (value: number | null) =>
      value === null ? null : formatDuration(value);
    const optionalCount = (value: number | null) =>
      value === null ? null : value.toLocaleString();

    return [
      {
        label: t("bangla.diagnostics.details.recordingDuration"),
        value: formatDuration(diagnostic.recording_duration_ms),
      },
      {
        label: t("bangla.diagnostics.details.recorderStop"),
        value: formatDuration(diagnostic.recorder_stop_ms),
      },
      {
        label: t("bangla.diagnostics.details.sttFinalize"),
        value: optionalDuration(diagnostic.stt_finalize_ms),
      },
      {
        label: t("bangla.diagnostics.details.sttTotal"),
        value: formatDuration(diagnostic.stt_ms),
      },
      {
        label: t("bangla.diagnostics.details.romanizationHeaders"),
        value: optionalDuration(diagnostic.romanization_headers_ms),
      },
      {
        label: t("bangla.diagnostics.details.romanizationBody"),
        value: optionalDuration(diagnostic.romanization_body_ms),
      },
      {
        label: t("bangla.diagnostics.details.providerQueue"),
        value: optionalDuration(diagnostic.provider_queue_ms),
      },
      {
        label: t("bangla.diagnostics.details.providerPrompt"),
        value: optionalDuration(diagnostic.provider_prompt_ms),
      },
      {
        label: t("bangla.diagnostics.details.providerCompletion"),
        value: optionalDuration(diagnostic.provider_completion_ms),
      },
      {
        label: t("bangla.diagnostics.details.providerTotal"),
        value: optionalDuration(diagnostic.provider_total_ms),
      },
      {
        label: t("bangla.diagnostics.details.promptTokens"),
        value: optionalCount(diagnostic.provider_prompt_tokens),
      },
      {
        label: t("bangla.diagnostics.details.outputTokens"),
        value: optionalCount(diagnostic.provider_output_tokens),
      },
      {
        label: t("bangla.diagnostics.details.thinkingTokens"),
        value: optionalCount(diagnostic.provider_thinking_tokens),
      },
      {
        label: t("bangla.diagnostics.details.pasteQueue"),
        value: formatDuration(diagnostic.paste_queue_ms),
      },
      {
        label: t("bangla.diagnostics.details.pasteCall"),
        value: formatDuration(diagnostic.paste_call_ms),
      },
      {
        label: t("bangla.diagnostics.details.recordingToTerminal"),
        value: formatDuration(diagnostic.recording_to_terminal_ms),
      },
      {
        label: t("bangla.diagnostics.details.outcomeCode"),
        value: diagnostic.outcome,
      },
      {
        label: t("bangla.diagnostics.details.errorCode"),
        value: diagnostic.error_code,
      },
      {
        label: t("bangla.diagnostics.details.fallbackReason"),
        value: diagnostic.fallback_reason,
      },
      {
        label: t("bangla.diagnostics.details.requestId"),
        value: diagnostic.provider_request_id,
      },
    ].filter(
      (row): row is { label: string; value: string } => row.value !== null,
    );
  }, [diagnostic, t]);

  const clearDiagnostic = async () => {
    setClearing(true);
    try {
      const result = await commands.clearLatestBanglaDiagnostic();
      if (result.status === "ok") {
        setDiagnostic(null);
        setShowDetails(false);
      }
    } finally {
      setClearing(false);
    }
  };

  const sttMetadata = diagnostic
    ? [
        providerLabel(diagnostic.stt_provider),
        diagnostic.stt_model,
        transportLabel(diagnostic.stt_transport),
      ]
        .filter(Boolean)
        .join(" · ")
    : "";
  const romanizationMetadata = diagnostic?.romanization_enabled
    ? [
        diagnostic.romanization_provider
          ? providerLabel(diagnostic.romanization_provider)
          : null,
        diagnostic.romanization_model,
      ]
        .filter(Boolean)
        .join(" · ")
    : t("bangla.diagnostics.disabled");

  return (
    <SettingsGroup
      title={t("bangla.diagnostics.title")}
      description={t("bangla.diagnostics.description")}
    >
      {!diagnostic ? (
        <p className="px-4 py-4 text-sm text-mid-gray">
          {t("bangla.diagnostics.empty")}
        </p>
      ) : (
        <div className="px-4 py-3">
          <div className="flex items-center justify-between gap-3 pb-2 border-b border-mid-gray/20">
            <span
              className={`px-2 py-1 rounded-md text-xs font-medium ${outcomeClasses[diagnostic.outcome_category]}`}
            >
              {t(outcomeTranslationKeys[diagnostic.outcome_category])}
            </span>
            <Button
              type="button"
              variant="danger-ghost"
              size="sm"
              onClick={() => void clearDiagnostic()}
              disabled={clearing}
              className="flex items-center gap-1.5"
              title={t("bangla.diagnostics.clear")}
              aria-label={t("bangla.diagnostics.clear")}
            >
              <Trash2 className="w-3.5 h-3.5" aria-hidden="true" />
              {t("bangla.diagnostics.clear")}
            </Button>
          </div>

          <div className="divide-y divide-mid-gray/20">
            <SummaryRow
              label={t("bangla.diagnostics.summary.total")}
              duration={formatDuration(diagnostic.post_stop_total_ms)}
            />
            <SummaryRow
              label={t("bangla.diagnostics.summary.transcription")}
              duration={formatDuration(diagnostic.stt_ms)}
              metadata={sttMetadata}
            />
            <SummaryRow
              label={t("bangla.diagnostics.summary.romanization")}
              duration={
                diagnostic.romanization_enabled
                  ? formatDuration(diagnostic.romanization_ms)
                  : undefined
              }
              metadata={romanizationMetadata}
            />
          </div>

          <button
            type="button"
            onClick={() => setShowDetails((visible) => !visible)}
            className="flex items-center justify-between w-full pt-3 text-xs font-medium text-text/70 hover:text-text transition-colors"
            aria-expanded={showDetails}
          >
            {showDetails
              ? t("bangla.diagnostics.hideDetails")
              : t("bangla.diagnostics.showDetails")}
            <ChevronDown
              className={`w-4 h-4 transition-transform ${showDetails ? "rotate-180" : ""}`}
              aria-hidden="true"
            />
          </button>

          {showDetails && (
            <div className="mt-3 pt-2 border-t border-mid-gray/20 divide-y divide-mid-gray/10">
              {details.map((row) => (
                <DetailRow key={row.label} {...row} />
              ))}
            </div>
          )}
        </div>
      )}
    </SettingsGroup>
  );
};
