package openai

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync"

	"github.com/router-for-me/CLIProxyAPI/v7/internal/registry"
	log "github.com/sirupsen/logrus"
)

type codexClientModelsPayload struct {
	Models []map[string]any `json:"models"`
}

type codexClientModelTemplateSource struct {
	name     string
	data     []byte
	required bool
}

var (
	codexClientModelTemplatesOnce sync.Once
	codexClientModelTemplates     map[string]map[string]any
	codexClientDefaultTemplate    map[string]any
	codexClientModelTemplatesErr  error
)

var codexClientAllowedReasoningLevels = map[string]struct{}{
	"none":   {},
	"low":    {},
	"medium": {},
	"high":   {},
	"xhigh":  {},
}

func (h *OpenAIAPIHandler) codexClientModelsResponse() map[string]any {
	return CodexClientModelsResponse(h.Models())
}

func CodexClientModelsResponse(models []map[string]any) map[string]any {
	return map[string]any{
		"models": buildCodexClientModels(models),
	}
}

func buildCodexClientModels(models []map[string]any) []map[string]any {
	templates, defaultTemplate, err := loadCodexClientModelTemplates()
	if err != nil || defaultTemplate == nil {
		return nil
	}

	result := make([]map[string]any, 0, len(models))
	for _, model := range models {
		id := strings.TrimSpace(stringModelValue(model, "id"))
		if id == "" {
			continue
		}

		if template, ok := templates[id]; ok {
			entry := cloneCodexClientModelMap(template)
			sanitizeCodexClientReasoningMetadata(entry)
			applyCodexClientVisibilityOverride(entry, id)
			result = append(result, entry)
			continue
		}

		entry := cloneCodexClientModelMap(defaultTemplate)
		applyCodexClientModelMetadata(entry, id, model)
		sanitizeCodexClientReasoningMetadata(entry)
		applyCodexClientVisibilityOverride(entry, id)
		result = append(result, entry)
	}

	sort.SliceStable(result, func(i, j int) bool {
		return codexClientModelPriority(result[i]) < codexClientModelPriority(result[j])
	})

	return result
}

func loadCodexClientModelTemplates() (map[string]map[string]any, map[string]any, error) {
	codexClientModelTemplatesOnce.Do(func() {
		codexClientModelTemplates, codexClientDefaultTemplate, codexClientModelTemplatesErr = loadCodexClientModelTemplatesFromSources(defaultCodexClientModelTemplateSources())
	})

	return codexClientModelTemplates, codexClientDefaultTemplate, codexClientModelTemplatesErr
}

func defaultCodexClientModelTemplateSources() []codexClientModelTemplateSource {
	sources := make([]codexClientModelTemplateSource, 0, 3)
	if data, err := readUserCodexModelsCache(); err == nil && len(strings.TrimSpace(string(data))) > 0 {
		sources = append(sources, codexClientModelTemplateSource{
			name: "user models_cache.json",
			data: data,
		})
	} else if err != nil && !os.IsNotExist(err) {
		log.Warnf("codex client models: failed to read user models_cache.json, using embedded fallback: %v", err)
	}

	sources = append(sources,
		codexClientModelTemplateSource{
			name:     "embedded models_cache.json",
			data:     registry.GetCodexModelsCacheJSON(),
			required: true,
		},
		codexClientModelTemplateSource{
			name:     "embedded codex_client_models.json",
			data:     registry.GetCodexClientModelsJSON(),
			required: true,
		},
	)
	return sources
}

func readUserCodexModelsCache() ([]byte, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return nil, err
	}
	return os.ReadFile(filepath.Join(home, ".codex", "models_cache.json"))
}

func loadCodexClientModelTemplatesFromSources(sources []codexClientModelTemplateSource) (map[string]map[string]any, map[string]any, error) {
	templates := make(map[string]map[string]any)
	var defaultTemplate map[string]any
	var firstRequiredErr error

	for _, source := range sources {
		if len(strings.TrimSpace(string(source.data))) == 0 {
			if source.required && firstRequiredErr == nil {
				firstRequiredErr = fmt.Errorf("%s is empty", source.name)
			}
			continue
		}

		var payload codexClientModelsPayload
		if err := json.Unmarshal(source.data, &payload); err != nil {
			if source.required && firstRequiredErr == nil {
				firstRequiredErr = fmt.Errorf("%s: %w", source.name, err)
			} else if !source.required {
				log.Warnf("codex client models: failed to parse %s, using fallback templates: %v", source.name, err)
			}
			continue
		}

		for _, model := range payload.Models {
			slug := strings.TrimSpace(stringModelValue(model, "slug"))
			if slug == "" {
				continue
			}
			if _, exists := templates[slug]; exists {
				if slug == "gpt-5.5" && defaultTemplate != nil {
					mergeMissingCodexClientModelFields(defaultTemplate, model)
				}
				continue
			}
			templates[slug] = cloneCodexClientModelMap(model)
			if slug == "gpt-5.5" && defaultTemplate == nil {
				defaultTemplate = cloneCodexClientModelMap(model)
			}
		}
	}

	if defaultTemplate == nil {
		if firstRequiredErr != nil {
			return nil, nil, firstRequiredErr
		}
		return nil, nil, fmt.Errorf("codex client model templates missing gpt-5.5 default")
	}

	return templates, defaultTemplate, nil
}

func mergeMissingCodexClientModelFields(target map[string]any, fallback map[string]any) {
	if target == nil {
		return
	}
	for key, value := range fallback {
		if _, exists := target[key]; exists {
			continue
		}
		target[key] = cloneCodexClientModelValue(value)
	}
}

func applyCodexClientModelMetadata(entry map[string]any, id string, model map[string]any) {
	info := registry.LookupModelInfo(id)

	displayName := stringModelValue(model, "display_name")
	description := stringModelValue(model, "description")
	contextWindow := intModelValue(model, "context_length")

	if info != nil {
		if info.DisplayName != "" {
			displayName = info.DisplayName
		}
		if info.Description != "" {
			description = info.Description
		}
		if info.ContextLength > 0 {
			contextWindow = info.ContextLength
		}
		if info.Type == registry.OpenAIImageModelType {
			entry["visibility"] = "hide"
		}
		applyCodexClientThinkingMetadata(entry, info.Thinking)
	}

	if displayName == "" {
		displayName = id
	}
	if description == "" {
		description = id
	}

	entry["slug"] = id
	entry["display_name"] = displayName
	entry["description"] = description
	entry["priority"] = 100
	entry["prefer_websockets"] = false
	delete(entry, "apply_patch_tool_type")
	delete(entry, "upgrade")
	delete(entry, "availability_nux")

	if contextWindow > 0 {
		entry["context_window"] = contextWindow
		entry["max_context_window"] = contextWindow
	}

	if baseInstructions := stringModelValue(model, "base_instructions"); baseInstructions != "" {
		entry["base_instructions"] = baseInstructions
	}
	if plans, ok := model["available_in_plans"]; ok {
		entry["available_in_plans"] = cloneCodexClientModelValue(plans)
	}
}

func applyCodexClientVisibilityOverride(entry map[string]any, id string) {
	switch strings.TrimSpace(id) {
	case "grok-imagine-image-quality", "gpt-image-2", "grok-imagine-image", "grok-imagine-video":
		entry["visibility"] = "hide"
	}
}

func applyCodexClientThinkingMetadata(entry map[string]any, thinking *registry.ThinkingSupport) {
	if thinking == nil || len(thinking.Levels) == 0 {
		return
	}

	levels := make([]any, 0, len(thinking.Levels))
	defaultLevel := ""
	firstLevel := ""
	for _, rawLevel := range thinking.Levels {
		level := normalizeCodexClientReasoningLevel(rawLevel)
		if level == "" {
			continue
		}
		if firstLevel == "" {
			firstLevel = level
		}
		if (defaultLevel == "" && level != "none") || level == "medium" {
			defaultLevel = level
		}
		levels = append(levels, map[string]any{
			"effort":      level,
			"description": codexClientReasoningDescription(level),
		})
	}
	if len(levels) == 0 {
		return
	}
	if defaultLevel == "" {
		defaultLevel = firstLevel
	}

	entry["supported_reasoning_levels"] = levels
	entry["default_reasoning_level"] = defaultLevel
}

func sanitizeCodexClientReasoningMetadata(entry map[string]any) {
	rawLevels, ok := entry["supported_reasoning_levels"].([]any)
	if !ok {
		return
	}

	levels := make([]any, 0, len(rawLevels))
	allowedDefaults := make(map[string]struct{}, len(rawLevels))
	for _, rawLevelEntry := range rawLevels {
		levelEntry, ok := rawLevelEntry.(map[string]any)
		if !ok {
			continue
		}
		level := normalizeCodexClientReasoningLevel(stringModelValue(levelEntry, "effort"))
		if level == "" {
			continue
		}
		clonedEntry := cloneCodexClientModelMap(levelEntry)
		clonedEntry["effort"] = level
		levels = append(levels, clonedEntry)
		allowedDefaults[level] = struct{}{}
	}

	if len(levels) == 0 {
		delete(entry, "supported_reasoning_levels")
		delete(entry, "default_reasoning_level")
		return
	}

	defaultLevel := normalizeCodexClientReasoningLevel(stringModelValue(entry, "default_reasoning_level"))
	if _, ok := allowedDefaults[defaultLevel]; !ok {
		defaultLevel = stringModelValue(levels[0].(map[string]any), "effort")
	}

	entry["supported_reasoning_levels"] = levels
	entry["default_reasoning_level"] = defaultLevel
}

func normalizeCodexClientReasoningLevel(rawLevel string) string {
	level := strings.ToLower(strings.TrimSpace(rawLevel))
	if _, ok := codexClientAllowedReasoningLevels[level]; !ok {
		return ""
	}
	return level
}

func codexClientReasoningDescription(level string) string {
	switch level {
	case "none":
		return "No reasoning"
	case "low":
		return "Fast responses with lighter reasoning"
	case "medium":
		return "Balances speed and reasoning depth for everyday tasks"
	case "high":
		return "Greater reasoning depth for complex problems"
	case "xhigh":
		return "Extra high reasoning depth for complex problems"
	default:
		return level
	}
}

func codexClientModelPriority(model map[string]any) int {
	if priority, ok := model["priority"].(int); ok {
		return priority
	}
	if priority, ok := model["priority"].(float64); ok {
		return int(priority)
	}
	return 100
}

func stringModelValue(model map[string]any, key string) string {
	if model == nil {
		return ""
	}
	value, ok := model[key]
	if !ok {
		return ""
	}
	if s, ok := value.(string); ok {
		return strings.TrimSpace(s)
	}
	return ""
}

func intModelValue(model map[string]any, key string) int {
	if model == nil {
		return 0
	}
	switch value := model[key].(type) {
	case int:
		return value
	case int64:
		return int(value)
	case float64:
		return int(value)
	default:
		return 0
	}
}

func cloneCodexClientModelMap(model map[string]any) map[string]any {
	if model == nil {
		return nil
	}
	cloned := make(map[string]any, len(model))
	for key, value := range model {
		cloned[key] = cloneCodexClientModelValue(value)
	}
	return cloned
}

func cloneCodexClientModelValue(value any) any {
	switch typed := value.(type) {
	case map[string]any:
		return cloneCodexClientModelMap(typed)
	case []any:
		cloned := make([]any, len(typed))
		for i, entry := range typed {
			cloned[i] = cloneCodexClientModelValue(entry)
		}
		return cloned
	case []string:
		return append([]string(nil), typed...)
	default:
		return value
	}
}
