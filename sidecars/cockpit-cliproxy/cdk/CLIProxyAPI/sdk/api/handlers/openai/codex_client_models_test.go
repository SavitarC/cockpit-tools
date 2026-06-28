package openai

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/router-for-me/CLIProxyAPI/v7/internal/registry"
)

func TestLoadCodexClientModelTemplatesMergesLocalBeforeEmbeddedFallback(t *testing.T) {
	localPayload := []byte(`{
		"models": [
			{
				"slug": "gpt-5.5",
				"display_name": "Local GPT-5.5",
				"priority": 1,
				"supported_reasoning_levels": [{"effort": "medium"}]
			},
			{
				"slug": "local-only",
				"display_name": "Local Only",
				"priority": 2,
				"supported_reasoning_levels": [{"effort": "low"}]
			}
		]
	}`)
	embeddedPayload := []byte(`{
		"models": [
			{
				"slug": "gpt-5.5",
				"display_name": "Embedded GPT-5.5",
				"priority": 9,
				"minimal_client_version": "0.124.0",
				"supported_reasoning_levels": [{"effort": "high"}]
			},
			{
				"slug": "fallback-only",
				"display_name": "Fallback Only",
				"priority": 10,
				"supported_reasoning_levels": [{"effort": "xhigh"}]
			}
		]
	}`)

	templates, defaultTemplate, err := loadCodexClientModelTemplatesFromSources([]codexClientModelTemplateSource{
		{name: "local models_cache.json", data: localPayload},
		{name: "embedded models_cache.json", data: embeddedPayload, required: true},
	})
	if err != nil {
		t.Fatalf("load templates failed: %v", err)
	}

	if got := stringModelValue(templates["gpt-5.5"], "display_name"); got != "Local GPT-5.5" {
		t.Fatalf("gpt-5.5 display_name = %q, want local template", got)
	}
	if got := stringModelValue(templates["gpt-5.5"], "minimal_client_version"); got != "" {
		t.Fatalf("gpt-5.5 exact template minimal_client_version = %q, want original local template", got)
	}
	if got := stringModelValue(defaultTemplate, "display_name"); got != "Local GPT-5.5" {
		t.Fatalf("default template display_name = %q, want local gpt-5.5", got)
	}
	if got := stringModelValue(defaultTemplate, "minimal_client_version"); got != "0.124.0" {
		t.Fatalf("default template minimal_client_version = %q, want embedded fallback field", got)
	}
	if got := stringModelValue(templates["fallback-only"], "display_name"); got != "Fallback Only" {
		t.Fatalf("fallback-only display_name = %q, want embedded fallback template", got)
	}
}

func TestLoadCodexClientModelTemplatesFallsBackWhenLocalIsEmpty(t *testing.T) {
	templates, defaultTemplate, err := loadCodexClientModelTemplatesFromSources([]codexClientModelTemplateSource{
		{name: "local models_cache.json", data: []byte(`{"models":[]}`)},
		{name: "embedded models_cache.json", data: []byte(`{
			"models": [
				{
					"slug": "gpt-5.5",
					"display_name": "Embedded GPT-5.5",
					"supported_reasoning_levels": [{"effort": "medium"}]
				}
			]
		}`), required: true},
	})
	if err != nil {
		t.Fatalf("load templates failed: %v", err)
	}

	if got := stringModelValue(templates["gpt-5.5"], "display_name"); got != "Embedded GPT-5.5" {
		t.Fatalf("gpt-5.5 display_name = %q, want embedded fallback template", got)
	}
	if got := stringModelValue(defaultTemplate, "display_name"); got != "Embedded GPT-5.5" {
		t.Fatalf("default template display_name = %q, want embedded gpt-5.5", got)
	}
}

func TestLoadCodexClientModelTemplatesUsesCompatibilityCatalogForExactMatches(t *testing.T) {
	templates, defaultTemplate, err := loadCodexClientModelTemplatesFromSources([]codexClientModelTemplateSource{
		{name: "embedded models_cache.json", data: []byte(`{
			"models": [
				{
					"slug": "gpt-5.5",
					"display_name": "Embedded GPT-5.5",
					"supported_reasoning_levels": [{"effort": "medium"}]
				}
			]
		}`), required: true},
		{name: "embedded codex_client_models.json", data: []byte(`{
			"models": [
				{
					"slug": "gpt-5.5",
					"display_name": "Compatibility GPT-5.5",
					"minimal_client_version": "0.124.0"
				},
				{
					"slug": "gpt-5.2",
					"display_name": "Compatibility GPT-5.2",
					"priority": 3
				}
			]
		}`), required: true},
	})
	if err != nil {
		t.Fatalf("load templates failed: %v", err)
	}

	if got := stringModelValue(templates["gpt-5.2"], "display_name"); got != "Compatibility GPT-5.2" {
		t.Fatalf("gpt-5.2 display_name = %q, want compatibility exact template", got)
	}
	if got := stringModelValue(defaultTemplate, "minimal_client_version"); got != "0.124.0" {
		t.Fatalf("default template minimal_client_version = %q, want compatibility fallback field", got)
	}
}

func TestDefaultCodexClientModelTemplateSourcesUseCompatibilityCatalogForExactFallback(t *testing.T) {
	t.Setenv("HOME", t.TempDir())

	templates, _, err := loadCodexClientModelTemplatesFromSources(defaultCodexClientModelTemplateSources())
	if err != nil {
		t.Fatalf("load templates failed: %v", err)
	}

	if got := stringModelValue(templates["gpt-5.2"], "display_name"); got == "" {
		t.Fatal("gpt-5.2 should be available from compatibility exact templates")
	}
}

func TestLoadCodexClientModelTemplatesUsesEmbeddedFallbackWhenUserCacheIsInvalid(t *testing.T) {
	tmpHome := t.TempDir()
	cacheDir := filepath.Join(tmpHome, ".codex")
	if err := os.MkdirAll(cacheDir, 0o700); err != nil {
		t.Fatalf("failed to create cache dir: %v", err)
	}
	if err := os.WriteFile(filepath.Join(cacheDir, "models_cache.json"), []byte("{invalid"), 0o600); err != nil {
		t.Fatalf("failed to write invalid cache: %v", err)
	}
	t.Setenv("HOME", tmpHome)

	templates, defaultTemplate, err := loadCodexClientModelTemplatesFromSources(defaultCodexClientModelTemplateSources())
	if err != nil {
		t.Fatalf("load templates failed: %v", err)
	}

	if got := stringModelValue(templates["gpt-5.5"], "display_name"); got != "GPT-5.5" {
		t.Fatalf("gpt-5.5 display_name = %q, want embedded fallback template", got)
	}
	if got := stringModelValue(defaultTemplate, "display_name"); got != "GPT-5.5" {
		t.Fatalf("default template display_name = %q, want embedded fallback template", got)
	}
}

func TestEmbeddedCodexModelsCacheIsValidFallback(t *testing.T) {
	templates, defaultTemplate, err := loadCodexClientModelTemplatesFromSources([]codexClientModelTemplateSource{
		{name: "embedded models_cache.json", data: registry.GetCodexModelsCacheJSON(), required: true},
	})
	if err != nil {
		t.Fatalf("load embedded models_cache.json failed: %v", err)
	}

	for _, slug := range []string{"gpt-5.5", "gpt-5.4", "gpt-5.3-codex-spark"} {
		if _, ok := templates[slug]; !ok {
			t.Fatalf("embedded models_cache.json missing %s", slug)
		}
	}
	if got := stringModelValue(defaultTemplate, "display_name"); got != "GPT-5.5" {
		t.Fatalf("default template display_name = %q, want GPT-5.5", got)
	}
}
