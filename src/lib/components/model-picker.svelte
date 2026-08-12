<script lang="ts">
	import { onMount } from "svelte";
	import Database from "@tauri-apps/plugin-sql";
	import ServerIcon from "@iconify-svelte/mage/server";
	import PlusIcon from "@iconify-svelte/mage/plus";
	import EditIcon from "@iconify-svelte/mage/edit";
	import TrashIcon from "@iconify-svelte/mage/trash";
	import CheckIcon from "@iconify-svelte/mage/check";
	import * as Sheet from "$lib/components/ui/sheet/index.js";
	import { Button, buttonVariants } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { cn } from "$lib/utils.js";

	const DB_PATH = "sqlite:settings.db";
	const CONFIGS_KEY = "model_configs";

	type ModelConfig = {
		id: string;
		name: string;
		baseUrl: string;
		apiKey: string;
		defaultModel: string;
		isDefault: boolean;
	};

	const emptyForm = {
		name: "",
		baseUrl: "",
		apiKey: "",
		defaultModel: "",
		isDefault: false,
	};

	let { open = $bindable(false) } = $props();

	let models = $state<ModelConfig[]>([]);
	let editing = $state<string | null>(null);
	let showForm = $state(false);
	let loading = $state(true);
	let saving = $state(false);
	let error = $state("");
	let status = $state("");
	let form = $state({ ...emptyForm });

	onMount(async () => {
		await load();
	});

	const load = async () => {
		loading = true;
		error = "";
		try {
			const db = await Database.load(DB_PATH);
			await db.execute(
				"CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)"
			);
			const rows = await db.select<{ value: string }[]>(
				"SELECT value FROM settings WHERE key = $1",
				[CONFIGS_KEY]
			);
			if (rows[0]?.value) {
				models = JSON.parse(rows[0].value) as ModelConfig[];
			} else {
				models = [];
			}
			await db.close();
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	};

	const persist = async (next: ModelConfig[]) => {
		const db = await Database.load(DB_PATH);
		await db.execute(
			"INSERT INTO settings (key, value) VALUES ($1, $2) ON CONFLICT(key) DO UPDATE SET value = $2",
			[CONFIGS_KEY, JSON.stringify(next)]
		);
		await db.close();
	};

	const openAdd = () => {
		editing = null;
		form = { ...emptyForm };
		showForm = true;
		error = "";
		status = "";
	};

	const openEdit = (m: ModelConfig) => {
		editing = m.id;
		form = {
			name: m.name,
			baseUrl: m.baseUrl,
			apiKey: m.apiKey,
			defaultModel: m.defaultModel,
			isDefault: m.isDefault,
		};
		showForm = true;
		error = "";
		status = "";
	};

	const saveForm = async () => {
		saving = true;
		error = "";
		status = "";
		try {
			const name = form.name.trim();
			const baseUrl = form.baseUrl.trim();
			const apiKey = form.apiKey.trim();
			const defaultModel = form.defaultModel.trim();
			if (!name || !baseUrl || !apiKey) {
				throw new Error("Name, API endpoint and API key are required");
			}

			let next: ModelConfig[];
			if (editing) {
				next = models.map((m) =>
					m.id === editing
						? {
								...m,
								name,
								baseUrl,
								apiKey,
								defaultModel,
								isDefault: form.isDefault,
							}
						: m
				);
			} else {
				next = [
					...models,
					{
						id: crypto.randomUUID(),
						name,
						baseUrl,
						apiKey,
						defaultModel,
						isDefault: form.isDefault,
					},
				];
			}

			const targetId = editing ?? next[next.length - 1]?.id;
			if (form.isDefault && targetId) {
				next = next.map((m) => ({ ...m, isDefault: m.id === targetId }));
			}

			await persist(next);
			models = next;
			showForm = false;
			status = editing ? "Model updated." : "Model added.";
		} catch (e) {
			error = String(e);
		} finally {
			saving = false;
		}
	};

	const setDefault = async (id: string) => {
		try {
			const next = models.map((m) => ({ ...m, isDefault: m.id === id }));
			await persist(next);
			models = next;
			status = "Default model updated.";
		} catch (e) {
			error = String(e);
		}
	};

	const removeModel = async (id: string) => {
		try {
			const next = models.filter((m) => m.id !== id);
			await persist(next);
			models = next;
			status = "Model removed.";
		} catch (e) {
			error = String(e);
		}
	};
</script>

<Sheet.Root bind:open>
	<Sheet.Trigger class={cn(buttonVariants({ variant: "outline" }), "gap-1.5")}>
		<ServerIcon class="size-4" />
		Model
	</Sheet.Trigger>

	<Sheet.Content side="right" class="sm:max-w-md">
		<Sheet.Header>
			<Sheet.Title>Model Configuration</Sheet.Title>
			<Sheet.Description>
				Manage model endpoints, API keys and choose the default model.
			</Sheet.Description>
		</Sheet.Header>

		{#if loading}
			<p class="px-4 text-sm text-muted-foreground">Loading…</p>
		{:else if showForm}
			<div class="flex flex-col gap-3 px-4">
				<div class="flex flex-col gap-2">
					<label for="model-name" class="text-sm font-medium">Name</label>
					<Input
						id="model-name"
						placeholder="DeepSeek"
						bind:value={form.name}
						autocomplete="off"
					/>
				</div>
				<div class="flex flex-col gap-2">
					<label for="model-endpoint" class="text-sm font-medium">API Endpoint</label>
					<Input
						id="model-endpoint"
						placeholder="https://api.deepseek.com/chat/completions"
						bind:value={form.baseUrl}
						autocomplete="off"
					/>
				</div>
				<div class="flex flex-col gap-2">
					<label for="model-api-key" class="text-sm font-medium">API Key</label>
					<Input
						id="model-api-key"
						type="password"
						placeholder="sk-…"
						bind:value={form.apiKey}
						autocomplete="off"
					/>
				</div>
				<div class="flex flex-col gap-2">
					<label for="model-id" class="text-sm font-medium">Default Model</label>
					<Input
						id="model-id"
						placeholder="deepseek-v4-flash"
						bind:value={form.defaultModel}
						autocomplete="off"
					/>
				</div>
				<label class="flex items-center gap-2 text-sm font-medium">
					<input type="checkbox" bind:checked={form.isDefault} class="size-4" />
					Use as default provider
				</label>
				<div class="flex gap-2">
					<Button onclick={saveForm} disabled={saving}>
						{saving ? "Saving…" : editing ? "Save changes" : "Add model"}
					</Button>
					<Button
						variant="outline"
						onclick={() => {
							showForm = false;
							error = "";
						}}
					>
						Cancel
					</Button>
				</div>
			</div>
		{:else}
			<div class="flex flex-col gap-3 px-4">
				{#if models.length === 0}
					<p class="text-sm text-muted-foreground">No models configured yet.</p>
				{:else}
					<div class="flex flex-col gap-2">
						{#each models as m (m.id)}
							<div class="flex items-start justify-between gap-2 rounded-lg border p-3">
								<div class="flex min-w-0 flex-col gap-0.5">
									<div class="flex items-center gap-2 text-sm font-medium">
										{m.name}
										{#if m.isDefault}
											<span
												class="rounded bg-primary/10 px-1.5 py-0.5 text-xs text-primary"
											>
												Default
											</span>
										{/if}
									</div>
									<p class="truncate text-xs text-muted-foreground">{m.baseUrl}</p>
									<p class="truncate text-xs text-muted-foreground">
										Model: {m.defaultModel || "—"}
									</p>
								</div>
								<div class="flex items-center gap-1">
									{#if !m.isDefault}
										<Button
											size="icon-sm"
											variant="ghost"
											title="Set as default"
											onclick={() => setDefault(m.id)}
										>
											<CheckIcon class="size-4" />
										</Button>
									{/if}
									<Button
										size="icon-sm"
										variant="ghost"
										title="Edit"
										onclick={() => openEdit(m)}
									>
										<EditIcon class="size-4" />
									</Button>
									<Button
										size="icon-sm"
										variant="ghost"
										class="text-destructive"
										title="Delete"
										onclick={() => removeModel(m.id)}
									>
										<TrashIcon class="size-4" />
									</Button>
								</div>
							</div>
						{/each}
					</div>
				{/if}
				<Button variant="outline" class="gap-1.5" onclick={openAdd}>
					<PlusIcon class="size-4" />
					Add model
				</Button>
			</div>
		{/if}

		<div class="px-4">
			{#if error}
				<p class="text-sm text-destructive">{error}</p>
			{/if}
			{#if status}
				<p class="text-sm text-muted-foreground">{status}</p>
			{/if}
		</div>

		<Sheet.Footer class="mt-4">
			<Sheet.Close>Close</Sheet.Close>
		</Sheet.Footer>
	</Sheet.Content>
</Sheet.Root>