import React, { useEffect, useState } from 'react';
import { useChunkingStore } from '@/stores/chunkingStore';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { CheckCircle, XCircle, AlertCircle, FileCode } from 'lucide-react';
import type { BusinessRule } from '@/types/chunking';

interface BusinessRuleValidatorProps {
  projectPath?: string;
}

export const BusinessRuleValidator: React.FC<BusinessRuleValidatorProps> = ({ projectPath }) => {
  const {
    pendingBusinessRules,
    isLoadingBusinessRules,
    fetchPendingBusinessRules,
    validateBusinessRule,
  } = useChunkingStore();

  const [selectedRule, setSelectedRule] = useState<BusinessRule | null>(null);
  const [ruleDescription, setRuleDescription] = useState('');
  const [userCorrection, setUserCorrection] = useState('');

  useEffect(() => {
    if (projectPath) {
      fetchPendingBusinessRules(projectPath);
    }
  }, [projectPath]);

  useEffect(() => {
    if (selectedRule) {
      setRuleDescription(selectedRule.ai_interpretation);
      setUserCorrection(selectedRule.user_correction || '');
    }
  }, [selectedRule]);

  const handleValidate = async (ruleId: number, approve: boolean) => {
    if (approve) {
      await validateBusinessRule(ruleId, ruleDescription, userCorrection || undefined);
    }
    setSelectedRule(null);
    setRuleDescription('');
    setUserCorrection('');
  };

  if (isLoadingBusinessRules) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-muted-foreground">Loading business rules...</div>
      </div>
    );
  }

  if (pendingBusinessRules.length === 0) {
    return (
      <Card>
        <CardContent className="pt-6">
          <div className="flex flex-col items-center justify-center h-64 text-muted-foreground">
            <CheckCircle className="h-12 w-12 mb-4 opacity-50" />
            <p>No pending business rules</p>
            <p className="text-sm">All rules have been validated</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="flex gap-4 h-full">
      {/* List of pending rules */}
      <div className="w-1/3">
        <Card className="h-full">
          <CardHeader>
            <CardTitle>Pending Rules ({pendingBusinessRules.length})</CardTitle>
            <CardDescription>Rules awaiting validation</CardDescription>
          </CardHeader>
          <CardContent>
            <ScrollArea className="h-[calc(100vh-300px)]">
              <div className="space-y-2">
                {pendingBusinessRules.map((rule) => (
                  <Card
                    key={rule.id}
                    className={`cursor-pointer transition-all ${
                      selectedRule?.id === rule.id ? 'ring-2 ring-primary' : ''
                    }`}
                    onClick={() => setSelectedRule(rule)}
                  >
                    <CardContent className="pt-4">
                      <div className="space-y-2">
                        <div className="flex items-start gap-2">
                          <FileCode className="h-4 w-4 mt-0.5 text-muted-foreground" />
                          <div className="flex-1 min-w-0">
                            <div className="font-medium truncate">{rule.entity_name}</div>
                            <div className="text-xs text-muted-foreground truncate">
                              {rule.file_path}
                            </div>
                          </div>
                        </div>
                        <div className="text-sm line-clamp-2">{rule.ai_interpretation}</div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </ScrollArea>
          </CardContent>
        </Card>
      </div>

      {/* Selected rule details */}
      <div className="flex-1">
        {selectedRule ? (
          <Card className="h-full flex flex-col">
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle>{selectedRule.entity_name}</CardTitle>
                  <CardDescription>{selectedRule.file_path}</CardDescription>
                </div>
                <Badge variant="secondary">
                  <AlertCircle className="h-3 w-3 mr-1" />
                  Pending
                </Badge>
              </div>
            </CardHeader>
            <CardContent className="flex-1 flex flex-col gap-4">
              <div>
                <label className="text-sm font-medium">AI Interpretation:</label>
                <Textarea
                  value={ruleDescription}
                  onChange={(e) => setRuleDescription(e.target.value)}
                  className="mt-1 min-h-[100px]"
                  placeholder="AI's interpretation of the business rule..."
                />
              </div>

              <div>
                <label className="text-sm font-medium">Your Correction (optional):</label>
                <Textarea
                  value={userCorrection}
                  onChange={(e) => setUserCorrection(e.target.value)}
                  className="mt-1 min-h-[100px]"
                  placeholder="Provide corrections or additional context..."
                />
              </div>

              <div className="flex gap-2 justify-end">
                <Button
                  onClick={() => handleValidate(selectedRule.id!, false)}
                  variant="outline"
                >
                  <XCircle className="h-4 w-4 mr-1" />
                  Reject
                </Button>
                <Button onClick={() => handleValidate(selectedRule.id!, true)}>
                  <CheckCircle className="h-4 w-4 mr-1" />
                  Approve
                </Button>
              </div>
            </CardContent>
          </Card>
        ) : (
          <Card>
            <CardContent className="pt-6">
              <div className="flex items-center justify-center h-64 text-muted-foreground">
                <p>Select a rule to validate</p>
              </div>
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  );
};
