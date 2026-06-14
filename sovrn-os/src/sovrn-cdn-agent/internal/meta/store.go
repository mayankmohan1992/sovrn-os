package meta

import (
	"encoding/json"
	"os"
	"path/filepath"
	"time"
)

type Store struct {
	dir string
}

func NewStore(dir string) *Store {
	os.MkdirAll(dir, 0755)
	return &Store{dir: dir}
}

func (s *Store) SavePushResult(result interface{}) error {
	data, err := json.Marshal(result)
	if err != nil {
		return err
	}
	filename := "push_" + time.Now().Format("20060102150405") + ".json"
	return os.WriteFile(filepath.Join(s.dir, filename), data, 0644)
}
