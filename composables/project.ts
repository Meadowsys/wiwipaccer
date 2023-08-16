import { invoke } from "@tauri-apps/api";

export async function invoke_add_recent_project(path: string) {
	await invoke("add_recent_project", { path });
}

export async function invoke_clear_recent_projects() {
	await invoke("clear_recent_projects");
}

export async function invoke_get_recent_projects() {
	return await invoke("get_recent_projects") as Array<[string, string]>;
}

export async function invoke_remove_recent_project(path: string) {
	await invoke("remove_recent_project", { path });
}

export async function invoke_open_project(path?: string) {
	if (path) await invoke("open_project", { path });
	else await invoke("open_project");
}