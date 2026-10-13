type ArtworkRecord = {
  thumbnail?: unknown;
  thumbnail_url?: unknown;
  cover?: unknown;
  banner?: unknown;
  videoId?: unknown;
  video_id?: unknown;
  trackId?: unknown;
  track_id?: unknown;
  id?: unknown;
  fallbackArtwork?: unknown;
};

const YOUTUBE_IMAGE_SIZES = [
  'maxresdefault.jpg',
  'hq720.jpg',
  'sddefault.jpg',
  'hqdefault.jpg',
  'mqdefault.jpg',
  'default.jpg',
];

function youtubeVideoId(source: string): string {
  return source.match(/(?:i|img)\.ytimg\.com\/vi(?:_webp)?\/([^/]+)\//i)?.[1] || '';
}

export function artworkUrl(source: unknown, size = 1600): string {
  const original = String(source || '');
  if (!original || original.startsWith('/')) return original;

  const youtubeId = youtubeVideoId(original);
  if (youtubeId && /\/(?:default|mqdefault|hqdefault|sddefault|hq720)\.(?:jpg|webp)(?:\?.*)?$/i.test(original)) {
    return original.replace(/\/(?:default|mqdefault|hqdefault|sddefault|hq720)\.(?:jpg|webp)(\?.*)?$/i, `/maxresdefault.jpg$1`);
  }

  return original
    .replace(/=w\d+-h\d+([^?&#]*)/g, `=w${size}-h${size}$1`)
    .replace(/=s\d+([^?&#]*)/g, `=s${size}$1`)
    .replace(/([?&](?:width|w)=)\d+/gi, `$1${size}`)
    .replace(/([?&](?:height|h)=)\d+/gi, `$1${size}`);
}

function youtubeRecordId(record: ArtworkRecord | null | undefined): string {
  const id = [record?.videoId, record?.video_id, record?.trackId, record?.track_id, record?.id]
    .map((value) => String(value || '').trim())
    .find((value) => /^[\w-]{11}$/.test(value));
  return id || '';
}

/** Resolve an artwork URL, deriving video art when metadata omitted its thumbnail. */
export function artworkFor(
  record: ArtworkRecord | null | undefined,
  size = 1600,
  fallbackArtwork: unknown = '',
): string {
  const source = [record?.thumbnail, record?.thumbnail_url, record?.cover, record?.banner]
    .map((value) => String(value || '').trim())
    .find((value) => value && !/placeholder\.(?:jpg|svg)$/i.test(value));
  if (source) return artworkUrl(source, size);

  const youtubeId = youtubeRecordId(record);
  if (youtubeId) return `https://i.ytimg.com/vi/${encodeURIComponent(youtubeId)}/maxresdefault.jpg`;

  const fallback = String(fallbackArtwork || record?.fallbackArtwork || '').trim();
  return fallback ? artworkUrl(fallback, size) : '/placeholder.svg';
}

function artworkCandidates(source: string, record: ArtworkRecord | null): string[] {
  const candidates: string[] = [];
  const sources = [source, record?.thumbnail, record?.thumbnail_url, record?.cover, record?.banner, record?.fallbackArtwork]
    .map((value) => String(value || '').trim())
    .filter(Boolean);
  const ids = [...sources.map(youtubeVideoId), youtubeRecordId(record)].filter(Boolean);

  for (const imageId of [...new Set(ids)]) {
    for (const size of YOUTUBE_IMAGE_SIZES) candidates.push(`https://i.ytimg.com/vi/${encodeURIComponent(imageId)}/${size}`);
  }

  for (const item of sources) {
    candidates.push(artworkUrl(item, 1600), artworkUrl(item, 1200), artworkUrl(item, 800), item);
  }
  candidates.push('/placeholder.svg');

  return [...new Set(candidates)];
}

export function recoverArtwork(event: Event, record: ArtworkRecord | null = null): void {
  const image = event.currentTarget as HTMLImageElement | null;
  if (!image) return;

  const source = String(
    record?.thumbnail || record?.thumbnail_url || record?.cover || record?.banner || image.dataset.originalArtwork || image.getAttribute('src') || '',
  );
  if (image.dataset.artworkRecoverySource !== source) {
    image.dataset.artworkRecoverySource = source;
    image.dataset.artworkRecoveryStep = '0';
  }

  const candidates = artworkCandidates(source, record);
  let step = Number(image.dataset.artworkRecoveryStep || 0);
  while (step < candidates.length) {
    const candidate = new URL(candidates[step], window.location.href).href;
    image.dataset.artworkRecoveryStep = String(++step);
    if (image.src !== candidate) {
      image.src = candidate;
      return;
    }
  }
}
