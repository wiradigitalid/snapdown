import React, { useCallback, useEffect, useState } from 'react';
import { FindingsEditor } from '@snapdown/ui';
import { deleteFinding, FindingDetailDto, listFindings, saveNote } from '../services/finding';

export const FindingsView: React.FC = () => {
  const [findings, setFindings] = useState<FindingDetailDto[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const fetchFindings = useCallback(async () => {
    try {
      const list = await listFindings();
      setFindings(list);
      setSelectedId((current) => {
        if (list.length > 0 && (!current || !list.some((f) => f.finding.id === current))) {
          return list[0].finding.id;
        }
        return current;
      });
    } catch (err) {
      console.error('Failed to list findings:', err);
    }
  }, []);

  useEffect(() => {
    fetchFindings();
  }, [fetchFindings]);

  const handleSaveNote = async (findingId: string, noteBody: string) => {
    await saveNote(findingId, noteBody);
    await fetchFindings();
  };

  const handleDeleteFinding = async (findingId: string) => {
    await deleteFinding(findingId);
    await fetchFindings();
  };

  return (
    <div data-testid="findings-view" style={{ padding: '16px' }}>
      <FindingsEditor
        findings={findings}
        selectedFindingId={selectedId}
        onSelectFinding={(id) => setSelectedId(id)}
        onSaveNote={handleSaveNote}
        onDeleteFinding={handleDeleteFinding}
      />
    </div>
  );
};
