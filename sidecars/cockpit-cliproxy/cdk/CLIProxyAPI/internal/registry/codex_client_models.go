package registry

import _ "embed"

//go:embed models/codex_client_models.json
var codexClientModelsJSON []byte

//go:embed models/models_cache.json
var codexModelsCacheJSON []byte

// GetCodexClientModelsJSON returns the embedded Codex client model catalog.
func GetCodexClientModelsJSON() []byte {
	return append([]byte(nil), codexClientModelsJSON...)
}

// GetCodexModelsCacheJSON returns the embedded Codex models_cache.json fallback.
func GetCodexModelsCacheJSON() []byte {
	return append([]byte(nil), codexModelsCacheJSON...)
}
