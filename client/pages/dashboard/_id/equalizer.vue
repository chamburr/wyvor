<template>
  <div>
    <BaseSelect
      v-model="preset"
      class="mb-4"
      placeholder="Select preset..."
      :allow-empty="false"
      :options="presetOptions"
    />
    <div class="d-lg-none">
      <div
        v-for="(element, index) in newBands"
        :key="`sm-${index}`"
        class="d-flex align-items-center"
      >
        <p class="equalizer-label mb-0">B{{ index + 1 }}</p>
        <BaseSlider
          v-model.number="newBands[index].gain"
          class="flex-grow-1"
          :range="{ min: -1, max: 1 }"
        />
      </div>
    </div>
    <div class="d-none d-lg-flex justify-content-between">
      <div
        v-for="(element, index) in newBands"
        :key="`lg-${index}`"
        class="d-flex flex-column align-items-center"
      >
        <BaseSlider
          v-model.number="newBands[index].gain"
          class="equalizer-control"
          :range="{ min: -1, max: 1 }"
          :options="{ orientation: 'vertical', direction: 'rtl' }"
        />
        <p class="equalizer-label text-center">B{{ index + 1 }}</p>
      </div>
    </div>
    <UpdateBar :changed="hasChanges" @save="saveBands" @reset="newBands = $api.clone(bands)" />
  </div>
</template>

<script>
import { mapGetters } from 'vuex'

export default {
  layout: 'dashboard',
  async asyncData({ $getContent }) {
    let presets = await $getContent('equalizer')
    presets = presets.map(element => {
      if (!element.bands) return element
      element.bands = JSON.parse(element.bands).map((element2, index) => ({
        band: index,
        gain: parseFloat(element2),
      }))
      return element
    })
    return {
      presets,
      preset: presets[0].name,
    }
  },
  data() {
    return {
      newBands: this.$api.clone(this.$store.getters['guild/player'].filters.equalizer.bands),
    }
  },
  computed: {
    presetOptions() {
      return this.presets.map(element => element.name)
    },
    hasChanges() {
      return !this.$api.isEqual(this.bands, this.newBands)
    },
    bands() {
      return this.player.filters.equalizer.bands
    },
    ...mapGetters('guild', ['player']),
  },
  watch: {
    preset(newValue) {
      const preset = this.presets.find(element => element.name === newValue)
      if (preset.bands) {
        this.newBands = this.$api.clone(preset.bands)
      }
    },
    newBands() {
      this.updatePreset()
    },
  },
  mounted() {
    this.updatePreset()
  },
  methods: {
    updatePreset() {
      let preset = this.presets.find(element => this.$api.isEqual(element.bands, this.newBands))
      if (!preset) {
        preset = this.presets.find(element => !element.bands)
      }

      this.preset = preset.name
    },
    async saveBands() {
      const bands = this.newBands.map(element => {
        if (element.gain < 0) element.gain = +(element.gain * 0.25).toFixed(2)
        return element
      })

      await this.$axios
        .$patch(`/guilds/${this.$route.params.id}/player`, { filters: { equalizer: { bands } } })
        .then(() => this.$toast.success('Updated the equalizer.'))
        .catch(this.$error)
    },
  },
}
</script>

<style scoped lang="scss">
.equalizer-label {
  width: 2.5em;
}
.equalizer-control {
  /deep/ .input-slider {
    height: 400px;
    max-height: 80vh;
  }
}
</style>
