import React, { useCallback, useEffect, useState, useMemo } from 'react';
import { convertFileSrc } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { FindingsEditor, FindingDetailItemDto } from '@snapdown/ui';
import {
  addMarker,
  deleteFinding,
  deleteMarker,
  FindingDetailDto,
  listFindings,
  saveNote,
  updateMarker,
} from '../services/finding';
import { getHotkeys, getSettings } from '../services/settings';
import { triggerOverlay } from '../services/capture';
import { OrphanReportView } from './OrphanReportView';

function resolveImagePath(imagePath: string, vaultPath: string): string {
  if (!imagePath) return '';
  if (
    imagePath.startsWith('/') ||
    imagePath.startsWith('\\') ||
    /^[a-zA-Z]:[\\/]/.test(imagePath)
  ) {
    return convertFileSrc(imagePath);
  }
  if (vaultPath) {
    const separator = vaultPath.endsWith('/') || vaultPath.endsWith('\\') ? '' : '/';
    return convertFileSrc(`${vaultPath}${separator}${imagePath}`);
  }
  return convertFileSrc(imagePath);
}

function renumberNoteAfterMarkerDelete(noteBody: string, deletedOrdinal: number): string {
  const lines = noteBody.split('\n');
  const result: string[] = [];

  for (const line of lines) {
    const match = line.match(/^(\s*)(\d+)(\..*)$/);
    if (match) {
      const indent = match[1];
      const num = parseInt(match[2], 10);
      const rest = match[3];

      if (num === deletedOrdinal) {
        // Line removed with marker
        continue;
      } else if (num > deletedOrdinal) {
        // Renumber contiguously
        result.push(`${indent}${num - 1}${rest}`);
      } else {
        result.push(line);
      }
    } else {
      result.push(line);
    }
  }

  return result.join('\n');
}

export interface FindingsViewProps {
  onCompose?: (selectedFindingIds: string[]) => void;
}

export const FindingsView: React.FC<FindingsViewProps> = ({ onCompose }) => {
  const [findings, setFindings] = useState<FindingDetailDto[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [vaultPath, setVaultPath] = useState<string>('');
  const [captureHotkey, setCaptureHotkey] = useState<string>('CommandOrControl+Shift+S');
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'findings' | 'orphan-report'>('findings');

  const fetchFindingsData = useCallback(async (autoSelectFirst = false) => {
    setIsLoading(true);
    setError(null);
    try {
      const [findingsList, settings, hotkeysDto] = await Promise.all([
        listFindings(),
        getSettings().catch(() => ({ vault_path: '', quality_budget: { max_long_edge: 1600, encoder_quality: 75 }, latest_finding_size: null })),
        getHotkeys().catch(() => ({ hotkeys: [], startup_warnings: [] })),
      ]);

      setFindings(findingsList);
      setVaultPath(settings.vault_path || '');

      const captureHk = hotkeysDto.hotkeys.find((h) => h.action === 'capture');
      if (captureHk && captureHk.shortcut) {
        setCaptureHotkey(captureHk.shortcut);
      }

      setSelectedId((current) => {
        if (findingsList.length > 0) {
          if (autoSelectFirst || !current || !findingsList.some((f) => f.finding.id === current)) {
            return findingsList[0].finding.id;
          }
          return current;
        }
        return null;
      });
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchFindingsData();

    let unlistenFn: (() => void) | undefined;
    try {
      const promise = listen('capture-completed', () => {
        fetchFindingsData(true);
      });
      promise.then((fn) => {
        unlistenFn = fn;
      }).catch(() => {});
    } catch {
      // Non-Tauri fallback
    }

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, [fetchFindingsData]);

  // Map findings to UI format with converted file src URLs
  const uiFindings: FindingDetailItemDto[] = useMemo(() => {
    return findings.map((f) => ({
      finding: {
        id: f.finding.id,
        image_path: f.finding.image_path,
        image_width: f.finding.image_width,
        image_height: f.finding.image_height,
        captured_at: f.finding.captured_at,
        source_monitor: f.finding.source_monitor,
        region: f.finding.region,
      },
      note: {
        id: f.note.id,
        finding_id: f.note.finding_id,
        body: f.note.body,
        updated_at: f.note.updated_at,
      },
      markers: f.markers.map((m) => ({
        id: m.id,
        finding_id: m.finding_id,
        ordinal: m.ordinal,
        x: m.x,
        y: m.y,
        comment: m.comment,
      })),
      imageSrc: resolveImagePath(f.finding.image_path, vaultPath),
    }));
  }, [findings, vaultPath]);

  const handleSaveNote = async (findingId: string, noteBody: string) => {
    await saveNote(findingId, noteBody);
    setFindings((prev) =>
      prev.map((item) =>
        item.finding.id === findingId
          ? { ...item, note: { ...item.note, body: noteBody } }
          : item
      )
    );
  };

  const handleDeleteFinding = async (findingId: string) => {
    await deleteFinding(findingId);
    await fetchFindingsData();
  };

  const handleAddMarker = async (findingId: string, x: number, y: number) => {
    const finding = findings.find((f) => f.finding.id === findingId);
    const existingMarkers = finding?.markers || [];
    const nextOrdinal = existingMarkers.length + 1;
    const markerId = crypto.randomUUID ? crypto.randomUUID() : `m_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`;
    const comment = `Marker ${nextOrdinal}`;

    await addMarker(findingId, markerId, x, y, comment);

    // Sync note line for the new marker
    if (finding) {
      const currentNote = finding.note.body;
      const linePattern = new RegExp(`^\\s*${nextOrdinal}\\..*$`, 'm');
      if (!linePattern.test(currentNote)) {
        const separator = currentNote.trim().length > 0 ? (currentNote.endsWith('\n') ? '' : '\n') : '';
        const updatedNote = `${currentNote}${separator}${nextOrdinal}. ${comment}`;
        await saveNote(findingId, updatedNote);
      }
    }

    await fetchFindingsData();
  };

  const handleUpdateMarkerPosition = async (
    findingId: string,
    markerId: string,
    x: number,
    y: number
  ) => {
    const finding = findings.find((f) => f.finding.id === findingId);
    const existingMarker = finding?.markers.find((m) => m.id === markerId);
    const comment = existingMarker?.comment || '';

    await updateMarker(findingId, markerId, x, y, comment);
    setFindings((prev) =>
      prev.map((f) =>
        f.finding.id === findingId
          ? {
              ...f,
              markers: f.markers.map((m) =>
                m.id === markerId ? { ...m, x, y } : m
              ),
            }
          : f
      )
    );
  };

  const handleDeleteMarker = async (findingId: string, markerId: string) => {
    const finding = findings.find((f) => f.finding.id === findingId);
    const markerToDelete = finding?.markers.find((m) => m.id === markerId);

    await deleteMarker(findingId, markerId);

    // Reverse SCN-04 sync: remove deleted marker line and renumber subsequent lines
    if (finding && markerToDelete) {
      const updatedNote = renumberNoteAfterMarkerDelete(finding.note.body, markerToDelete.ordinal);
      if (updatedNote !== finding.note.body) {
        await saveNote(findingId, updatedNote);
      }
    }

    await fetchFindingsData();
  };

  const handleCaptureClick = async () => {
    try {
      await triggerOverlay();
    } catch (err) {
      console.error('Failed to trigger capture overlay:', err);
    }
  };

  if (viewMode === 'orphan-report') {
    return (
      <div data-testid="findings-view" style={{ width: '100%', height: '100%' }}>
        <OrphanReportView onBack={() => { setViewMode('findings'); fetchFindingsData(); }} />
      </div>
    );
  }

  return (
    <div data-testid="findings-view" style={{ width: '100%', height: '100%' }}>
      <FindingsEditor
        findings={uiFindings}
        selectedFindingId={selectedId}
        isLoading={isLoading}
        error={error}
        captureHotkey={captureHotkey}
        onSelectFinding={(id) => setSelectedId(id)}
        onSaveNote={handleSaveNote}
        onDeleteFinding={handleDeleteFinding}
        onAddMarker={handleAddMarker}
        onUpdateMarkerPosition={handleUpdateMarkerPosition}
        onDeleteMarker={handleDeleteMarker}
        onOpenOrphanReport={() => setViewMode('orphan-report')}
        onCompose={onCompose}
        onCaptureClick={handleCaptureClick}
        onRetry={() => fetchFindingsData(false)}
      />
    </div>
  );
};
