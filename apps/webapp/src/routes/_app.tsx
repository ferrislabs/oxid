import { createFileRoute, Outlet } from "@tanstack/react-router";
import { Bell, Search } from "lucide-react";

import { AppSidebar } from "#/components/app-sidebar";
import { Button } from "#/components/ui/button";
import {
	SidebarInset,
	SidebarProvider,
	SidebarTrigger,
} from "#/components/ui/sidebar";

export const Route = createFileRoute("/_app")({ component: AppLayout });

function AppLayout() {
	return (
		<SidebarProvider>
			<AppSidebar />
			<SidebarInset>
				<header className="sticky top-0 z-10 flex h-16 shrink-0 items-center gap-3 border-b bg-background/80 px-3 backdrop-blur md:px-6">
					<SidebarTrigger className="-ml-1" />

					<div className="relative flex max-w-xl flex-1 items-center">
						<Search className="pointer-events-none absolute left-3 size-4 text-muted-foreground" />
						<input
							type="search"
							placeholder="Rechercher…"
							className="h-10 w-full rounded-xl border bg-card pl-9 pr-16 text-sm outline-none transition-colors placeholder:text-muted-foreground focus:border-orange-400 focus:ring-2 focus:ring-orange-100 dark:focus:ring-orange-500/20"
						/>
						<kbd className="pointer-events-none absolute right-3 hidden h-5 select-none items-center gap-0.5 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground sm:inline-flex">
							⌘K
						</kbd>
					</div>

					<div className="ml-auto flex items-center gap-2">
						<span className="hidden items-center gap-1.5 rounded-lg border bg-card px-3 py-1.5 text-xs font-medium text-muted-foreground sm:inline-flex">
							espace:{" "}
							<span className="font-mono text-foreground">production</span>
						</span>
						<Button
							variant="ghost"
							size="icon"
							className="rounded-lg text-muted-foreground"
						>
							<Bell />
							<span className="sr-only">Notifications</span>
						</Button>
						<button
							type="button"
							className="flex size-9 items-center justify-center rounded-lg bg-orange-600 text-sm font-semibold text-white shadow-sm transition-opacity hover:opacity-90"
						>
							NB
						</button>
					</div>
				</header>
				<div className="flex flex-1 flex-col">
					<Outlet />
				</div>
			</SidebarInset>
		</SidebarProvider>
	);
}
