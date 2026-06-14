package cdn

import (
	"log"
	"os"
	"path/filepath"
)

// PushContent pushes a file to all configured CDN providers
func PushContent(cacheDir, cid, localPath string, providers []Provider) error {
	for _, p := range providers {
		if !p.IsEnabled() {
			continue
		}
		log.Printf("Pushing %s to %s", cid, p.Name())
		if err := p.Push(cid, localPath, 0, ""); err != nil {
			log.Printf("Failed to push %s to %s: %v", cid, p.Name(), err)
			continue
		}
	}
	return nil
}

// StoreLocal copies content to local cache
func StoreLocal(cacheDir, cid, srcPath string) error {
	dstDir := filepath.Join(cacheDir, cid[:2])
	if err := os.MkdirAll(dstDir, 0755); err != nil {
		return err
	}
	dstPath := filepath.Join(dstDir, cid)
	src, err := os.Open(srcPath)
	if err != nil {
		return err
	}
	defer src.Close()

	dst, err := os.Create(dstPath)
	if err != nil {
		return err
	}
	defer dst.Close()

	_, err = dst.ReadFrom(src)
	return err
}
