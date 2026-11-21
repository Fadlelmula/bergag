import * as Dialog from "@radix-ui/react-dialog";
import { LoaderCircle, Plus, Trash2, X } from "lucide-react";
import { useState } from "react";
import {
  actionLabel,
  conditionLabel,
  defaultRuleDefinition,
  newRuleMatchMode,
  parseRuleDefinition,
  TRUST_LEVELS,
} from "../../application/discoveryService";
import { Button } from "../../components/ui/Button";
import { IconButton } from "../../components/ui/IconButton";
import type {
  Feed,
  Label,
  Rule,
  RuleAction,
  RuleCondition,
  RuleDefinition,
  TrustLevel,
} from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";

interface RuleBuilderDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  rule: Rule | null;
  feeds: Feed[];
  labels: Label[];
  onSaved: () => void;
}

const CONDITION_OPTIONS: Array<{ field: RuleCondition["field"]; label: string }> = [
  { field: "titleContains", label: "Title contains" },
  { field: "summaryContains", label: "Summary contains" },
  { field: "contentContains", label: "Content contains" },
  { field: "feedIs", label: "Is from feed" },
  { field: "feedTitleContains", label: "Feed title contains" },
  { field: "categoryIs", label: "Category is" },
  { field: "trustAtLeast", label: "Trust level at least" },
  { field: "hasLabel", label: "Has label" },
  { field: "isRead", label: "Read state" },
  { field: "isSaved", label: "Saved state" },
  { field: "isFavorite", label: "Favorite state" },
  { field: "isArchived", label: "Archive state" },
  { field: "isQueued", label: "Queue state" },
];

const ACTION_OPTIONS: Array<{ action: RuleAction["action"]; label: string }> = [
  { action: "markRead", label: "Read state" },
  { action: "setSaved", label: "Read later" },
  { action: "setFavorite", label: "Favorite" },
  { action: "setArchived", label: "Archive" },
  { action: "setQueued", label: "Reading queue" },
  { action: "addLabel", label: "Add label" },
  { action: "removeLabel", label: "Remove label" },
];

function makeCondition(field: RuleCondition["field"]): RuleCondition {
  switch (field) {
    case "feedIs":
      return { field, feedId: "" };
    case "trustAtLeast":
      return { field, value: "medium" };
    case "hasLabel":
      return { field, labelId: "" };
    case "isRead":
    case "isSaved":
    case "isFavorite":
    case "isArchived":
    case "isQueued":
      return { field, value: true };
    default:
      return { field, value: "" } as RuleCondition;
  }
}

function makeAction(action: RuleAction["action"]): RuleAction {
  switch (action) {
    case "addLabel":
    case "removeLabel":
      return { action, labelId: "" };
    default:
      return { action, value: true } as RuleAction;
  }
}

function conditionValue(condition: RuleCondition): string {
  switch (condition.field) {
    case "feedIs":
      return condition.feedId;
    case "hasLabel":
      return condition.labelId;
    case "trustAtLeast":
      return condition.value;
    default:
      return "value" in condition ? String(condition.value) : "";
  }
}

function booleanCondition(condition: RuleCondition): boolean {
  return "value" in condition && condition.value === true;
}

function actionValue(action: RuleAction): string {
  if (action.action === "addLabel" || action.action === "removeLabel") return action.labelId;
  return String(action.value);
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function RuleBuilderDialog({
  open,
  onOpenChange,
  rule,
  feeds,
  labels,
  onSaved,
}: RuleBuilderDialogProps) {
  const initial = rule
    ? (parseRuleDefinition(rule.rule_json) ?? defaultRuleDefinition())
    : defaultRuleDefinition();
  const [name, setName] = useState(rule?.name ?? "");
  const [description, setDescription] = useState(rule?.description ?? "");
  const [enabled, setEnabled] = useState(rule?.enabled ?? true);
  const [isSmartFeed, setIsSmartFeed] = useState(rule?.is_smart_feed ?? false);
  const [matchMode, setMatchMode] = useState(initial.matchMode);
  const [conditions, setConditions] = useState<RuleCondition[]>(initial.conditions);
  const [actions, setActions] = useState<RuleAction[]>(initial.actions);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const updateCondition = (index: number, condition: RuleCondition) => {
    setConditions((current) => current.map((item, at) => (at === index ? condition : item)));
  };

  const updateAction = (index: number, action: RuleAction) => {
    setActions((current) => current.map((item, at) => (at === index ? action : item)));
  };

  const save = async () => {
    setBusy(true);
    setError(null);
    const definition: RuleDefinition = { matchMode, conditions, actions };
    try {
      if (rule) {
        await desktop.updateRule(
          rule.id,
          name,
          description.trim() || null,
          definition,
          enabled,
          isSmartFeed,
        );
      } else {
        await desktop.createRule(
          name,
          description.trim() || null,
          definition,
          enabled,
          isSmartFeed,
        );
      }
      onSaved();
      onOpenChange(false);
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="bergag-dialog-overlay" />
        <Dialog.Content
          className="bergag-dialog"
          style={{ width: "min(760px, calc(100vw - 48px))", height: "auto" }}
        >
          <div className="bergag-dialog__body">
            <div className="bergag-dialog__header">
              <div>
                <Dialog.Title className="bergag-title bergag-title--small">
                  {rule ? "Edit rule" : "New rule"}
                </Dialog.Title>
                <Dialog.Description className="bergag-subtitle">
                  Rules run locally against your stored articles. Nothing is sent anywhere.
                </Dialog.Description>
              </div>
              <Dialog.Close asChild>
                <IconButton label="Close" icon={<X size={15} aria-hidden="true" />} />
              </Dialog.Close>
            </div>
            <div className="bergag-dialog__content bergag-rule-builder">
              <div className="bergag-field">
                <label className="bergag-field__label" htmlFor="rule-name">
                  Rule name
                </label>
                <input
                  id="rule-name"
                  className="bergag-input"
                  value={name}
                  maxLength={64}
                  onChange={(event) => setName(event.target.value)}
                  autoFocus
                />
              </div>
              <div className="bergag-field">
                <label className="bergag-field__label" htmlFor="rule-description">
                  Description
                </label>
                <input
                  id="rule-description"
                  className="bergag-input"
                  value={description}
                  maxLength={500}
                  placeholder="Optional note about what this rule does"
                  onChange={(event) => setDescription(event.target.value)}
                />
              </div>
              <div className="bergag-rule-flags">
                <label className="bergag-checkbox">
                  <input
                    type="checkbox"
                    checked={enabled}
                    onChange={(event) => setEnabled(event.target.checked)}
                  />
                  Enabled
                </label>
                <label className="bergag-checkbox">
                  <input
                    type="checkbox"
                    checked={isSmartFeed}
                    onChange={(event) => setIsSmartFeed(event.target.checked)}
                  />
                  Show as smart feed
                </label>
                <label className="bergag-field bergag-field--inline">
                  <span className="bergag-field__label">Match</span>
                  <select
                    className="bergag-select"
                    value={matchMode}
                    onChange={(event) => setMatchMode(newRuleMatchMode(event.target.value))}
                  >
                    <option value="all">All conditions</option>
                    <option value="any">Any condition</option>
                  </select>
                </label>
              </div>

              <section className="bergag-rule-section" aria-label="Rule conditions">
                <div className="bergag-rule-section__header">
                  <h3 className="bergag-title bergag-title--tiny">Conditions</h3>
                  <Button
                    variant="ghost"
                    size="sm"
                    icon={<Plus size={13} />}
                    disabled={conditions.length >= 20}
                    onClick={() =>
                      setConditions((current) => [...current, makeCondition("titleContains")])
                    }
                  >
                    Add condition
                  </Button>
                </div>
                {conditions.map((condition, index) => (
                  <div className="bergag-rule-row" key={`condition-${index}`}>
                    <select
                      className="bergag-select"
                      aria-label={`Condition ${index + 1} field`}
                      value={condition.field}
                      onChange={(event) =>
                        updateCondition(
                          index,
                          makeCondition(event.target.value as RuleCondition["field"]),
                        )
                      }
                    >
                      {CONDITION_OPTIONS.map((option) => (
                        <option key={option.field} value={option.field}>
                          {option.label}
                        </option>
                      ))}
                    </select>
                    {condition.field === "feedIs" ? (
                      <select
                        className="bergag-select"
                        aria-label={`Condition ${index + 1} feed`}
                        value={condition.feedId}
                        onChange={(event) =>
                          updateCondition(index, { field: "feedIs", feedId: event.target.value })
                        }
                      >
                        <option value="">Choose a feed</option>
                        {feeds.map((feed) => (
                          <option key={feed.id} value={feed.id}>
                            {feed.title}
                          </option>
                        ))}
                      </select>
                    ) : condition.field === "hasLabel" ? (
                      <select
                        className="bergag-select"
                        aria-label={`Condition ${index + 1} label`}
                        value={condition.labelId}
                        onChange={(event) =>
                          updateCondition(index, { field: "hasLabel", labelId: event.target.value })
                        }
                      >
                        <option value="">Choose a label</option>
                        {labels.map((label) => (
                          <option key={label.id} value={label.id}>
                            {label.name}
                          </option>
                        ))}
                      </select>
                    ) : condition.field === "trustAtLeast" ? (
                      <select
                        className="bergag-select"
                        aria-label={`Condition ${index + 1} trust level`}
                        value={condition.value}
                        onChange={(event) =>
                          updateCondition(index, {
                            field: "trustAtLeast",
                            value: event.target.value as TrustLevel,
                          })
                        }
                      >
                        {TRUST_LEVELS.map((level) => (
                          <option key={level.value} value={level.value}>
                            {level.label}
                          </option>
                        ))}
                      </select>
                    ) : condition.field === "isRead" ||
                      condition.field === "isSaved" ||
                      condition.field === "isFavorite" ||
                      condition.field === "isArchived" ||
                      condition.field === "isQueued" ? (
                      <select
                        className="bergag-select"
                        aria-label={`Condition ${index + 1} value`}
                        value={booleanCondition(condition) ? "true" : "false"}
                        onChange={(event) =>
                          updateCondition(index, {
                            ...condition,
                            value: event.target.value === "true",
                          })
                        }
                      >
                        <option value="true">Yes</option>
                        <option value="false">No</option>
                      </select>
                    ) : (
                      <input
                        className="bergag-input"
                        aria-label={`Condition ${index + 1} value`}
                        value={conditionValue(condition)}
                        maxLength={200}
                        placeholder="Text to match"
                        onChange={(event) =>
                          updateCondition(index, { ...condition, value: event.target.value })
                        }
                      />
                    )}
                    <IconButton
                      label={`Remove condition ${index + 1}`}
                      icon={<Trash2 size={13} aria-hidden="true" />}
                      disabled={conditions.length <= 1}
                      onClick={() =>
                        setConditions((current) => current.filter((_, at) => at !== index))
                      }
                    />
                  </div>
                ))}
              </section>

              <section className="bergag-rule-section" aria-label="Rule actions">
                <div className="bergag-rule-section__header">
                  <h3 className="bergag-title bergag-title--tiny">Actions</h3>
                  <Button
                    variant="ghost"
                    size="sm"
                    icon={<Plus size={13} />}
                    disabled={actions.length >= 20}
                    onClick={() => setActions((current) => [...current, makeAction("markRead")])}
                  >
                    Add action
                  </Button>
                </div>
                {actions.map((action, index) => (
                  <div className="bergag-rule-row" key={`action-${index}`}>
                    <select
                      className="bergag-select"
                      aria-label={`Action ${index + 1} kind`}
                      value={action.action}
                      onChange={(event) =>
                        updateAction(index, makeAction(event.target.value as RuleAction["action"]))
                      }
                    >
                      {ACTION_OPTIONS.map((option) => (
                        <option key={option.action} value={option.action}>
                          {option.label}
                        </option>
                      ))}
                    </select>
                    {action.action === "addLabel" || action.action === "removeLabel" ? (
                      <select
                        className="bergag-select"
                        aria-label={`Action ${index + 1} label`}
                        value={action.labelId}
                        onChange={(event) =>
                          updateAction(index, {
                            action: action.action,
                            labelId: event.target.value,
                          })
                        }
                      >
                        <option value="">Choose a label</option>
                        {labels.map((label) => (
                          <option key={label.id} value={label.id}>
                            {label.name}
                          </option>
                        ))}
                      </select>
                    ) : (
                      <select
                        className="bergag-select"
                        aria-label={`Action ${index + 1} value`}
                        value={actionValue(action)}
                        onChange={(event) =>
                          updateAction(index, {
                            ...action,
                            value: event.target.value === "true",
                          })
                        }
                      >
                        <option value="true">{actionLabel({ ...action, value: true })}</option>
                        <option value="false">{actionLabel({ ...action, value: false })}</option>
                      </select>
                    )}
                    <IconButton
                      label={`Remove action ${index + 1}`}
                      icon={<Trash2 size={13} aria-hidden="true" />}
                      disabled={actions.length <= 1}
                      onClick={() =>
                        setActions((current) => current.filter((_, at) => at !== index))
                      }
                    />
                  </div>
                ))}
              </section>

              <div className="bergag-rule-preview">
                {conditions
                  .map((condition) => conditionLabel(condition))
                  .join(matchMode === "all" ? " AND " : " OR ")}
                {" → "}
                {actions.map((action) => actionLabel(action)).join(", ")}
              </div>

              {error ? <p className="bergag-error">{error}</p> : null}
              <div className="bergag-inline-actions">
                <Button
                  variant="primary"
                  onClick={() => void save()}
                  disabled={busy || !name.trim() || conditions.length === 0 || actions.length === 0}
                  icon={busy ? <LoaderCircle size={14} /> : undefined}
                >
                  {rule ? "Save rule" : "Create rule"}
                </Button>
                <Dialog.Close asChild>
                  <Button variant="ghost">Cancel</Button>
                </Dialog.Close>
              </div>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
