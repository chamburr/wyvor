<template>
  <div id="player-container">
    <div class="d-flex mb-5">
      <BaseSelect
        v-model="searchResult"
        class="flex-grow-1"
        placeholder="Search..."
        label="title"
        :internal-search="false"
        :options="searchOptions"
        :loading="searchLoading"
        @search-change="searchTrack"
      />
      <BaseButton type="primary" class="ml-3 flex-shrink-0" @click="addTrack">
        Add to Queue
      </BaseButton>
    </div>
    <div class="d-flex flex-wrap flex-md-nowrap justify-content-between align-items-center mt-5">
      <h2 class="mb-0">Queue</h2>
      <div>
        <BaseButton v-b-toggle="'player-queue'" type="primary" size="sm">
          {{ historyVisible ? 'Hide History' : 'Show History' }}
        </BaseButton>
        <BaseButton type="danger" size="sm" @click="clearQueue">Clear</BaseButton>
      </div>
    </div>
    <BCollapse id="player-queue" v-model="historyVisible" role="tabpanel">
      <Track
        v-for="element in history"
        :key="JSON.stringify(element)"
        v-bind="element"
        @remove="removeTrack(element.index)"
        @move="moveTrack(element.index)"
        @jump="setPlaying(element.index)"
      />
    </BCollapse>
    <Track v-if="playing != null" v-bind="playing" :playing="true" />
    <Track
      v-for="element in nextUp"
      :key="JSON.stringify(element)"
      v-bind="element"
      @remove="removeTrack(element.index)"
      @move="moveTrack(element.index)"
      @jump="setPlaying(element.index)"
    />
    <div v-if="tracks.length === 0" class="text-center mt-4">
      <span class="display-2">:(</span>
      <p class="h5 mt-4">No tracks in the queue.</p>
    </div>
    <h2 class="mt-5 mb-2">Player</h2>
    <div id="player-player" class="bg-gray pb-3 pb-md-5 pt-3 position-sticky">
      <Player
        :position="player.position"
        :volume="player.volume"
        :paused="player.paused"
        :looping="player.looping"
        v-bind="playing"
        @shuffle="shuffleQueue"
        @backward="setPlaying(playing.index - 1)"
        @paused="setPaused($event)"
        @forward="setPlaying(playing.index + 1)"
        @looping="setLooping($event)"
        @volume="setVolume($event)"
        @position="setPosition($event)"
      />
    </div>
  </div>
</template>

<script>
import { mapGetters } from 'vuex'

export default {
  layout: 'dashboard',
  data() {
    return {
      searchResult: null,
      searchLoading: false,
      searchOptions: [],
      searchThrottle: '',
      historyVisible: false,
    }
  },
  computed: {
    tracks() {
      return this.queue.map((element, index) => ({
        index,
        track: { ...element },
        user: {
          id: element.author,
          username: element.username,
          discriminator: element.discriminator,
        },
      }))
    },
    history() {
      return this.tracks.filter(element => element.index < this.player.playing)
    },
    playing() {
      return this.tracks.find(element => element.index === this.player.playing)
    },
    nextUp() {
      return this.tracks.filter(element => element.index > this.player.playing)
    },
    ...mapGetters('guild', ['player', 'queue']),
  },
  methods: {
    async addTrack() {
      if (!this.searchResult) {
        this.$toast.danger('No track is selected.')
        return
      }
      await this.$axios
        .$post(`/guilds/${this.$route.params.id}/queue`, {
          track: this.searchResult.track,
        })
        .then(() => {
          this.searchResult = null
          this.$toast.success('Track added.')
        })
        .catch(this.$error)
    },
    async removeTrack(index) {
      await this.$axios
        .$delete(`/guilds/${this.$route.params.id}/queue/${index}`)
        .then(() => this.$toast.success('Track removed.'))
        .catch(this.$error)
    },
    async moveTrack(index) {
      await this.$axios
        .$put(`/guilds/${this.$route.params.id}/queue/${index}/position`, { position: index - 1 })
        .then(() => this.$toast.success('Track moved up.'))
        .catch(this.$error)
    },
    searchTrack(query) {
      clearTimeout(this.searchThrottle)

      if (query === '') {
        this.searchOptions = []
        this.searchLoading = false
        return
      }

      this.searchLoading = true
      this.searchThrottle = setTimeout(async () => {
        this.searchOptions = await this.$axios
          .$get('/tracks', { params: { query } })
          .catch(this.$error)
        this.searchLoading = false
      }, 200)
    },
    async shuffleQueue() {
      await this.$axios
        .$post(`/guilds/${this.$route.params.id}/queue/shuffle`)
        .then(() => this.$toast.success('Shuffled the queue.'))
        .catch(this.$error)
    },
    async clearQueue() {
      await this.$modal('Do you really want to clear the queue?', {
        title: 'Confirmation',
        okTitle: 'Clear',
        okVariant: 'danger',
      }).then(async res => {
        if (!res) return
        await this.$axios
          .$delete(`/guilds/${this.$route.params.id}/queue`)
          .then(() => this.$toast.success('Cleared the queue.'))
          .catch(this.$error)
      })
    },
    async setPlaying(playing) {
      await this.$axios
        .$patch(`/guilds/${this.$route.params.id}/player`, { playing })
        .then(() => this.$toast.success('Changed the playing track.'))
        .catch(this.$error)
    },
    async setPaused(paused) {
      await this.$axios
        .$patch(`/guilds/${this.$route.params.id}/player`, { paused })
        .then(() => this.$toast.success(`${paused ? 'Paused' : 'Resumed'} the player.`))
        .catch(this.$error)
    },
    async setLooping(looping) {
      await this.$axios
        .$patch(`/guilds/${this.$route.params.id}/player`, { looping })
        .then(() =>
          this.$toast.success(
            looping === 'none' ? 'Disabled looping.' : `Changed to loop the ${looping}.`
          )
        )
        .catch(this.$error)
    },
    async setVolume(volume) {
      await this.$axios
        .$patch(`/guilds/${this.$route.params.id}/player`, { volume })
        .then(() => this.$toast.success('Changed the volume of the player.'))
        .catch(err => {
          this.$error(err)
          const volume = this.player.volume
          this.player.volume = -1
          this.player.volume = volume
        })
    },
    async setPosition(position) {
      await this.$axios
        .$patch(`/guilds/${this.$route.params.id}/player`, { position })
        .then(() => this.$toast.success('Changed the position of the player.'))
        .catch(err => {
          this.$error(err)
          const position = this.player.position
          this.player.position = -1
          this.player.position = position
        })
    },
  },
}
</script>

<style scoped lang="scss">
@include media-breakpoint-up(md) {
  #player-container {
    margin-bottom: -3em;
  }
}

#player-player {
  bottom: 0;
}
</style>
