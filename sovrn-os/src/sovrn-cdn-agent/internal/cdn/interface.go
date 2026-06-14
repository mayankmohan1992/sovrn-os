package cdn

// Provider defines the interface for CDN providers
type Provider interface {
	Name() string
	Push(cid, localPath string, size int64, checksum string) error
	Pull(cid, localPath string) error
	Status(pushID string) (string, error)
	IsEnabled() bool
}
