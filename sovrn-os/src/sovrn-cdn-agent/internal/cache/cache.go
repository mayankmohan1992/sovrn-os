package cache

import (
	"os"
	"path/filepath"
)

// Cache manages local content-addressed storage
type Cache struct {
	dir string
}

// New creates a new cache instance
func New(dir string) *Cache {
	os.MkdirAll(dir, 0755)
	return &Cache{dir: dir}
}

// Get retrieves content from cache by CID
func (c *Cache) Get(cid string) ([]byte, error) {
	path := c.path(cid)
	return os.ReadFile(path)
}

// Put stores content in cache with CID as key
func (c *Cache) Put(cid string, data []byte) error {
	dir := filepath.Join(c.dir, cid[:2])
	if err := os.MkdirAll(dir, 0755); err != nil {
		return err
	}
	return os.WriteFile(c.path(cid), data, 0644)
}

// Has checks if content exists in cache
func (c *Cache) Has(cid string) bool {
	_, err := os.Stat(c.path(cid))
	return err == nil
}

// Path returns the local file path for a CID
func (c *Cache) Path(cid string) string {
	return c.path(cid)
}

func (c *Cache) path(cid string) string {
	return filepath.Join(c.dir, cid[:2], cid)
}
