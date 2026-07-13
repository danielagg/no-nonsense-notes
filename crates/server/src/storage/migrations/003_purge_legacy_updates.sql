-- Greenfield cleanup: only blobs prefixed with the current note-type byte are valid.
-- Old direct Loro blobs and the retired empty-blob deletion marker are disposable.
DELETE FROM updates
WHERE length(blob) = 0
   OR hex(substr(blob, 1, 1)) NOT IN ('00', '01');
