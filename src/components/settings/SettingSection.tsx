import { useId, type ReactNode } from 'react';

/** Shared presentation only; field bindings and save behavior live in each section. */
export function SettingSection({ id, title, description, children, className = '' }: {
  id?: string;
  title: string;
  description?: string;
  children: ReactNode;
  className?: string;
}) {
  const headingId = useId();
  return (
    <section id={id} className={`setting-section ${className}`} aria-labelledby={headingId} tabIndex={-1}>
      <header className="setting-section-heading">
        <h2 id={headingId}>{title}</h2>
        {description && <p>{description}</p>}
      </header>
      <div className="setting-section-body">{children}</div>
    </section>
  );
}
