<template>
  <div>
    <h2 class="mt-4 text-center">{{ lyrics.title ? lyrics.title.trim() : ':(' }}</h2>
    <p id="lyrics-content" class="text-center">
      {{ lyrics.content || 'No lyrics found for the current song.' }}
    </p>
  </div>
</template>

<script>
import { mapGetters } from 'vuex'

export default {
  layout: 'dashboard',
  async asyncData({ $axios, $fatal, store }) {
    const track = store.getters['guild/queue'][store.getters['guild/player'].playing]
    if (!track) {
      return {
        lyrics: {},
      }
    }

    const lyrics = await $axios
      .$get(`/tracks/lyrics`, { params: { id: encodeURIComponent(track.track) } })
      .then(res => res)
      .catch(err => {
        if (err.response.status !== 404) $fatal(err)
      })

    return {
      lyrics: lyrics || {},
    }
  },
  computed: {
    playing() {
      return this.queue[this.player.playing]
    },
    ...mapGetters('guild', ['player', 'queue']),
  },
  watch: {
    playing(newValue, oldValue) {
      if (newValue && oldValue && newValue.track !== oldValue.track) {
        this.$nuxt.refresh()
      }
    },
  },
}
</script>

<style scoped>
#lyrics-content {
  white-space: pre-line;
}
</style>
