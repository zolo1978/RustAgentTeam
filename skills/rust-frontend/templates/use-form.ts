// hooks/useForm.ts — generic form state management hook
import { useState, useCallback } from 'react';

export interface FormState<T> {
  values: T;
  errors: Partial<Record<keyof T, string>>;
  isSubmitting: boolean;
  serverError: string | null;
}

export function useForm<T extends Record<string, unknown>>(options: {
  initialValues: T;
  validate: (values: T) => Partial<Record<keyof T, string>>;
  onSubmit: (values: T) => Promise<void>;
}) {
  const { initialValues, validate, onSubmit } = options;
  const [state, setState] = useState<FormState<T>>({
    values: initialValues, errors: {}, isSubmitting: false, serverError: null,
  });

  const handleChange = useCallback(
    (field: keyof T, value: T[keyof T]) =>
      setState((s) => {
        const { [field]: _, ...rest } = s.errors;
        return { ...s, values: { ...s.values, [field]: value }, errors: rest as typeof s.errors };
      }),
    [],
  );

  const handleSubmit = useCallback(
    async (e?: React.FormEvent) => {
      e?.preventDefault();
      const errors = validate(state.values);
      if (Object.keys(errors).length > 0) { setState((s) => ({ ...s, errors })); return; }
      setState((s) => ({ ...s, isSubmitting: true, serverError: null }));
      try { await onSubmit(state.values); } catch (err) {
        setState((s) => ({
          ...s, isSubmitting: false,
          serverError: err instanceof Error ? err.message : '操作失败，请稍后重试',
        }));
      }
    },
    [state.values, validate, onSubmit],
  );

  const reset = useCallback(
    () => setState({ values: initialValues, errors: {}, isSubmitting: false, serverError: null }),
    [initialValues],
  );

  return { ...state, handleChange, handleSubmit, reset };
}
