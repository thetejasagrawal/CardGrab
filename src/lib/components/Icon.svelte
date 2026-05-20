<script lang="ts">
  import {
    Archive,
    ArrowDownWideNarrow,
    ArrowUpNarrowWide,
    Camera,
    Check,
    Clock,
    File,
    Folder,
    Image,
    LayoutGrid,
    List,
    MemoryStick,
    Music,
    Play,
    Search,
    Settings,
    SlidersHorizontal,
  } from '@lucide/svelte';
  import type { Component } from 'svelte';

  export type IconName =
    | 'archive'
    | 'camera'
    | 'check'
    | 'clock'
    | 'folder'
    | 'gear'
    | 'grid'
    | 'list'
    | 'music'
    | 'photo'
    | 'play'
    | 'search'
    | 'sd-card'
    | 'sliders'
    | 'sort-asc'
    | 'sort-desc'
    | 'file';

  const icons: Record<IconName, Component<any>> = {
    archive: Archive,
    camera: Camera,
    check: Check,
    clock: Clock,
    folder: Folder,
    gear: Settings,
    grid: LayoutGrid,
    list: List,
    music: Music,
    photo: Image,
    play: Play,
    search: Search,
    'sd-card': MemoryStick,
    sliders: SlidersHorizontal,
    'sort-asc': ArrowUpNarrowWide,
    'sort-desc': ArrowDownWideNarrow,
    file: File,
  };

  let {
    name,
    size = 15,
    stroke = 1.75,
    title,
  } = $props<{
    name: IconName;
    size?: number;
    stroke?: number;
    title?: string;
  }>();

  let IconComponent = $derived(icons[name] ?? File);
</script>

<IconComponent
  class="icon"
  size={size}
  strokeWidth={stroke}
  aria-hidden={title ? undefined : 'true'}
  aria-label={title}
/>
