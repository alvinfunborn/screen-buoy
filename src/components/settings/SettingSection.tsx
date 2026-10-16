import type { ReactNode } from 'react';

/** Shared presentation only; field bindings and save behavior live in each section. */
export function SettingSection({ title, description, children, className = '' }: {
  title: string;
  description?: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <section className={`setting-section ${className}`}>
      <header className="setting-section-heading">
        <h3>{title}</h3>
        {description && <p>{description}</p>}
      </header>
      <div className="setting-section-body">{children}</div>
    </section>
  );
}
