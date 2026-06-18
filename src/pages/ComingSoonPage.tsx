interface ComingSoonPageProps {
  label: string;
}

export default function ComingSoonPage({ label }: ComingSoonPageProps) {
  return (
    <div className="page">
      <h1>{label}</h1>
      <p className="page-subtitle">This test module isn't built yet. Contributions welcome!</p>
    </div>
  );
}
