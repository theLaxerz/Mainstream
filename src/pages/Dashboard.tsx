import { Clock } from "../components/Clock";
import { EmailSection } from "../components/EmailSection";
import { MailSection } from "../components/MailSection";
import { FinanceSection } from "../components/FinanceSection";
import { HealthSection } from "../components/HealthSection";
import { HomeSection } from "../components/HomeSection";
import { MessagesSection } from "../components/MessagesSection";
import { NewsSection } from "../components/NewsSection";
import { NotesSection } from "../components/NotesSection";
import { ShortcutsSection } from "../components/ShortcutsSection";
import { StreamingSection } from "../components/StreamingSection";
import { YouTubeSection } from "../components/YouTubeSection";
import { requestDashboardRefresh } from "../lib/refresh";
import "./Dashboard.css";

export function Dashboard() {
  return (
    <div className="dashboard">
      <header className="dashboard-hero">
        <p className="brand">Mainstream</p>
        <Clock />
        <p className="hero-tagline">Your day, gathered in one calm place.</p>
        <button
          type="button"
          className="btn btn-ghost dashboard-refresh"
          onClick={() => requestDashboardRefresh()}
        >
          Refresh all
        </button>
      </header>

      <div className="dashboard-grid">
        <MessagesSection />
        <EmailSection />
        <MailSection />
        <NewsSection />
        <FinanceSection />
        <NotesSection />
        <HealthSection />
        <HomeSection />
        <YouTubeSection />
        <StreamingSection />
        <ShortcutsSection />
      </div>
    </div>
  );
}
