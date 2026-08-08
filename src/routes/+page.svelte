<script lang="ts">
  import * as Breadcrumb from "$lib/components/ui/breadcrumb/index.js";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import { Textarea } from "$lib/components/ui/textarea";
  import SidebarLeft from "$lib/components/sidebar-left.svelte";
  import SidebarRight from "$lib/components/sidebar-right.svelte";
  import { Button } from "$lib/components/ui/button";
</script>

<Sidebar.Provider>
  <SidebarLeft />
  <Sidebar.Inset>
    <header
      class="sticky top-0 flex h-14 shrink-0 items-center gap-2 bg-background"
    >
      <div class="flex flex-1 items-center gap-2 px-3">
        <Sidebar.Trigger />
        <Separator
          orientation="vertical"
          class="me-2 data-[orientation=vertical]:h-4"
        />
        <Breadcrumb.Root>
          <Breadcrumb.List>
            <Breadcrumb.Item>
              <Breadcrumb.Page class="line-clamp-1">
                Project Management & Task Tracking
              </Breadcrumb.Page>
            </Breadcrumb.Item>
          </Breadcrumb.List>
        </Breadcrumb.Root>
      </div>
    </header>
    <div class="flex flex-1 flex-col gap-4 p-4">
      <div class="mx-auto h-auto w-full max-w-3xl rounded-xl bg-muted/50 px-4 py-3">
        <Textarea placeholder="What would you like to do today?" autofocus class="resize-none focus-visible:border-0 focus-visible:ring-0 border-none outline-0 shadow-none" />
        <div class="flex mt-2 justify-end">
          <Button>Ask</Button>
        </div>
      </div>
      <div
        class="mx-auto h-[100vh] w-full max-w-3xl rounded-xl bg-muted/50"
      ></div>
    </div>
  </Sidebar.Inset>
  <SidebarRight />
</Sidebar.Provider>
